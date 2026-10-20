use arterm::client_config;
#[path = "../client_protocol.rs"]
mod client_protocol;
#[path = "../client_output.rs"]
mod client_output;
#[path = "../file_protocol.rs"]
mod file_protocol;

use anyhow::{bail, ensure, Context, Result};
use arterm::statusln as eprintln;
use client_config::{ClientConfig, Target};
use arterm::{
    console::{Console, Terminal},
    deployment,
    engine::{End, Engine, SessionRejected},
    local_control::{self, Operation, Owner},
    store::{self, SessionReference, Store},
    transport::{ensure_authenticated, login, Forward, LoginRequired, TunnelLink, DEFAULT_CONNECTION_TIMEOUT},
};
use std::{
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::Duration,
};
use uuid::Uuid;

const DEFAULT_RETRIES: u32 = 5;

#[derive(Default)]
struct ConnectionFlags {
    address: Option<String>,
    stdio: bool,
    retries: Option<u32>,
    timeout: Option<Duration>,
    shell: Option<String>,
    cwd: Option<String>,
}

impl ConnectionFlags {
    fn connection_timeout(&self) -> Duration {
        self.timeout.unwrap_or(DEFAULT_CONNECTION_TIMEOUT)
    }
}

fn usage() {
    println!(
        "arTerm client {}\n\
Usage:\n\
  arterm setup [--devtunnel-path PATH] [--no-download]\n\
  arterm --login | login | logout\n\
  arterm add ALIAS --tunnel NAME-OR-ID --host-path ABSOLUTE\n\
  arterm list [--json]\n\
  arterm remove ALIAS\n\
  arterm connect ALIAS [REF] [--shell EXE] [--cwd PATH] [--retries 0..20] [--timeout 60s]\n\
  arterm resume ALIAS REF [--retries 0..20] [--timeout 60s]\n\
  arterm list --client [--json] | list --server MACHINE [--json] [--timeout 60s]\n\
  arterm send MACHINE SESSION --command TEXT [--command-id UUID] [--timeout 60s] [--wait] [--json]\n\
  arterm send MACHINE SESSION --file LOCALPATH [--json]\n\
  arterm receive MACHINE SESSION --file ABSOLUTE-REMOTEPATH [--json]\n\
  arterm read MACHINE SESSION [--lines N | --command-id UUID] [--json]\n\
  arterm interrupt MACHINE SESSION [--json] | detach MACHINE SESSION [--json]\n\
  arterm terminate MACHINE SESSION [--json] [--timeout 60s]\n\
  arterm doctor [ALIAS [--timeout 60s]]\n\
  arterm --help | --version | --privacy\n\n\
Privacy: read packaged PRIVACY.md or arterm --privacy before downloads/sign-in.\n\
Client Setup initializes local files. setup is optional repair/custom-path configuration, not sign-in.\n\
Network commands renew positively missing/expired sign-in once; use arterm --login for interactive GitHub sign-in with the host's account.\n\
list shows local session references without network or sign-in; completed/proven nonresumable sessions are retired.\n\
Offline, disconnected, and unconfirmed references are retained; remote existence is not inferred from network failure.\n\
Connecting a retired session name creates fresh identity/state; an old GUID never resolves to its replacement.\n\
Use list --server MACHINE for current authorized remote inventory, including detached running sessions.\n\
Without REF, connect only prints a reusable command. Names are not passwords.\n\
Host aliases are canonical; resume and GUID lookup never create. Fresh authorized clients need no copied recovery files.\n\
Tunnel startup defaults to 60s per attempt; connect, doctor ALIAS, list --server and terminate accept --timeout in positive whole seconds (e.g. 120s).\n\
This covers authentication checks, discovery and control-port readiness, not the session lifetime, host handshake or interactive sign-in recovery.\n\
Control commands print readable results by default; use --json for structured automation output.\n\
Commands require a compatible, command-enabled PowerShell session. Existing sessions are not retrofitted.\n\
Send waits up to 30s for readiness by default; --timeout overrides it. --wait requires --timeout and shares its budget with completion.\n\
Command IDs are retained for the session lifetime; capacity is 256, with no silent eviction or replacement.\n\
Local IPC requires OS trust, the pinned certificate, and byte-identical client builds; cross-elevation is rejected.\n\
Transfers use unique session TEMP destinations, not terminal paste. An explicit ZIP file stays a file.\n\
All received files, including extracted folder files, are automatically unblocked before publication; sources are unchanged.\n\
This is not a malware scan, safety verdict, execution-policy change, or an open-file-lock unlock.\n\
Directory sources require directory-transfer-zip-v1; independent archive review remains a release blocker.\n\
Ctrl+] detaches without terminating the remote session.",
        env!("CARGO_PKG_VERSION")
    );
}

fn value(args: &[String], index: &mut usize, flag: &str) -> Result<String> {
    *index += 1;
    let value = args
        .get(*index)
        .with_context(|| format!("{flag} requires a value"))?;
    ensure!(!value.is_empty(), "{flag} requires a non-empty value");
    Ok(value.clone())
}

fn parse_connection_flags(args: &[String], allow_create: bool) -> Result<ConnectionFlags> {
    let mut flags = ConnectionFlags::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--address" => flags.address = Some(value(args, &mut index, "--address")?),
            "--stdio" => flags.stdio = true,
            "--timeout" if flags.timeout.is_none() => {
                let text = value(args, &mut index, "--timeout")?;
                let seconds: u64 = text.strip_suffix('s')
                    .context("connection timeout must be whole seconds, e.g. 60s")?
                    .parse().context("invalid connection timeout")?;
                ensure!(seconds > 0, "connection timeout must be positive");
                let timeout = Duration::from_secs(seconds);
                ensure!(std::time::Instant::now().checked_add(timeout).is_some(),
                    "connection timeout exceeds supported clock range");
                flags.timeout = Some(timeout);
            }
            "--retries" => {
                let retries: u32 = value(args, &mut index, "--retries")?
                    .parse()
                    .context("invalid retry count")?;
                ensure!(retries <= 20, "retry count must be 0..20");
                flags.retries = Some(retries);
            }
            "--shell" if allow_create => flags.shell = Some(value(args, &mut index, "--shell")?),
            "--cwd" if allow_create => {
                let cwd = value(args, &mut index, "--cwd")?;
                ensure!(Path::new(&cwd).is_absolute(), "remote cwd must be absolute");
                flags.cwd = Some(cwd);
            }
            other => bail!("unknown option: {other}"),
        }
        index += 1;
    }
    if let Some(address) = &flags.address {
        let endpoint: std::net::SocketAddr = address.parse().context("invalid --address")?;
        ensure!(endpoint.ip().is_loopback(), "--address must be loopback");
    }
    Ok(flags)
}

#[cfg(test)]
#[test]
fn connection_timeout_defaults_and_overrides_are_validated() {
    assert_eq!(parse_connection_flags(&[], true).unwrap().connection_timeout(), Duration::from_secs(60));
    let args = vec!["--timeout".into(), "120s".into()];
    assert_eq!(parse_connection_flags(&args, true).unwrap().connection_timeout(), Duration::from_secs(120));
    let (flags, json) = output_connection_flags(&["--json".into(), "--timeout".into(), "2s".into()]).unwrap();
    assert!(json);
    assert_eq!(flags.connection_timeout(), Duration::from_secs(2));
    for value in ["0s", "-1s", "1.5s", "NaNs", "60", "1m", "18446744073709551615s"] {
        let args = vec!["--timeout".into(), value.into()];
        assert!(parse_connection_flags(&args, true).is_err(), "{value}");
    }
    assert!(parse_connection_flags(&["--timeout".into()], true).is_err());
    assert!(parse_connection_flags(&[
        "--timeout".into(), "60s".into(), "--timeout".into(), "120s".into(),
    ], true).is_err());
}

fn devtunnel_path(config: &ClientConfig) -> Result<&Path> {
    config
        .devtunnel_path
        .as_deref()
        .context("client is not set up; run arterm setup")
}

fn run_vendor(path: &Path, args: &[&str]) -> Result<()> {
    let status = Command::new(path)
        .args(args)
        .status()
        .with_context(|| format!("start {}", path.display()))?;
    ensure!(status.success(), "{} exited with {status}", path.display());
    Ok(())
}

fn setup(root: &Path, args: &[String]) -> Result<()> {
    eprintln!("{}", deployment::PRIVACY_NOTICE);
    let mut override_path = None;
    let mut no_download = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--devtunnel-path" => {
                override_path = Some(PathBuf::from(value(args, &mut index, "--devtunnel-path")?))
            }
            "--no-download" => no_download = true,
            other => bail!("unknown setup option: {other}"),
        }
        index += 1;
    }
    if let Some(path) = &override_path {
        ensure!(path.is_absolute(), "--devtunnel-path must be absolute");
    }
    let selected = client_config::initialize(root, override_path.as_deref(), no_download)?;
    println!("Local client ready with {}. Use arterm login if not signed in, then arterm add to register a host. Existing sign-in was not changed.", selected.display());
    Ok(())
}

fn add_target(root: &Path, args: &[String]) -> Result<()> {
    let alias = args.first().context("add requires an alias")?;
    let mut tunnel = None;
    let mut host_path = None;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--tunnel" => tunnel = Some(value(args, &mut index, "--tunnel")?),
            "--host-path" => host_path = Some(value(args, &mut index, "--host-path")?),
            other => bail!("unknown add option: {other}"),
        }
        index += 1;
    }
    let tunnel = tunnel.as_deref().context("add requires --tunnel")?;
    let host_path = host_path.as_deref().context("add requires --host-path")?;
    client_config::validate_registration(alias, tunnel, host_path)?;
    let target = client_config::update(root, |config| {
        client_config::add(config, alias, tunnel, host_path)
    })?;
    println!(
        "Added {alias} -> {} ({})",
        target.tunnel_id, target.host_path
    );
    Ok(())
}

fn list_targets(root: &Path, args: &[String]) -> Result<()> {
    ensure!(args.is_empty() || args == ["--json"],
        "list accepts only --json, --client or --server MACHINE");
    let config = client_config::load(root)?;
    let mut machines = Vec::new();
    for (alias, target) in &config.targets {
        let dir = client_config::state_dir(root, target);
        let sessions = store::active_sessions(&dir, &target.target_id)
            .map_err(|_| anyhow::anyhow!("invalid local session mapping or recovery metadata; inventory unavailable"))?;
        machines.push(serde_json::json!({
            "machine": alias, "tunnel": target.tunnel_id, "host_path": target.host_path,
            "sessions": sessions,
        }));
    }
    println!("{}", client_output::registered(&machines, !args.is_empty())?);
    Ok(())
}

fn remove_target(root: &Path, args: &[String]) -> Result<()> {
    ensure!(args.len() == 1, "remove requires exactly one alias");
    client_config::validate_alias(&args[0])?;
    client_config::update(root, |config| client_config::remove(config, root, &args[0]))?;
    println!("Removed alias {}. Recovery records were retained.", args[0]);
    Ok(())
}

fn connect_link(
    config: &ClientConfig,
    target: &Target,
    address: Option<&str>,
    timeout: Duration,
) -> Result<TunnelLink> {
    if let Some(address) = address {
        return TunnelLink::connect(address, &target.host_path, None);
    }
    let (forward, address) = Forward::start_with_timeout(devtunnel_path(config)?, &target.tunnel_id, timeout)?;
    TunnelLink::connect(&address, &target.host_path, Some(forward))
}

fn probe_retirement(config: &ClientConfig, target: &Target, address: Option<&str>, timeout: Duration,
    state: &store::State) -> Result<Option<store::RetirementReason>> {
    ensure!(state.origin.is_some(), "no saved broker identity; remote existence remains unknown");
    let mut link = connect_link(config, target, address, timeout)?;
    client_protocol::retirement_from_inventory(&mut link, state)
}

fn terminate_remote(config: &ClientConfig, target: &Target, address: Option<&str>, timeout: Duration,
    state: &store::State) -> Result<client_protocol::Termination> {
    if state.mode == store::SessionMode::CreatePending {
        if state.ended {
            if let Ok(Some(reason)) = probe_retirement(config, target, address, timeout, state) {
                return Ok(client_protocol::Termination::Retired(reason));
            }
        }
        bail!("creation not confirmed; remote existence remains unknown and reference was retained");
    }
    ensure!(state.origin.is_some(), "no saved broker identity; reference retained");
    let outcome = {
        let mut link = connect_link(config, target, address, timeout)?;
        client_protocol::terminate(&mut link, state)?
    };
    if outcome != client_protocol::Termination::Rejected { return Ok(outcome); }
    match probe_retirement(config, target, address, timeout, state) {
        Ok(Some(reason)) => Ok(client_protocol::Termination::Retired(reason)),
        _ => bail!("saved session identity rejected; remote existence remains unconfirmed and reference was retained"),
    }
}

fn termination_response(state: &mut store::State, outcome: client_protocol::Termination) -> Result<serde_json::Value> {
    Ok(match outcome {
        client_protocol::Termination::Confirmed => {
            state.mark_ended(true)?;
            serde_json::json!({"status":"terminated", "session_id":state.id})
        }
        client_protocol::Termination::AcceptedUnconfirmed => {
            state.mark_ended(false)?;
            serde_json::json!({"status":"termination_accepted", "session_id":state.id})
        }
        client_protocol::Termination::Retired(reason) => {
            state.mark_retired(reason)?;
            serde_json::json!({"status":"retired", "session_id":state.id,"retirement_reason":reason})
        }
        client_protocol::Termination::Rejected => unreachable!("unreconciled rejection"),
    })
}

fn already_retired(id: Uuid, reason: store::RetirementReason) -> serde_json::Value {
    serde_json::json!({"status":"already_retired","session_id":id,"retirement_reason":reason})
}

#[cfg(test)]
#[test]
fn accepted_unconfirmed_rejects_contradictory_exit_flags_without_mutation_or_completion() {
    let mut state = store::State::new("fixture".into(), "powershell.exe".into(), vec![], None);
    state.mode = store::SessionMode::Attached;
    state.origin = Some(vec![7; 16]);
    state.exit_confirmed = Some(true);
    let before = serde_json::to_vec(&state).unwrap();
    assert!(termination_response(&mut state, client_protocol::Termination::AcceptedUnconfirmed).is_err());
    assert!(serde_json::to_vec(&state).unwrap() == before);
    assert!(!state.ended && state.retirement_reason.is_none());
    state.exit_confirmed = None;
    let response = termination_response(&mut state, client_protocol::Termination::AcceptedUnconfirmed).unwrap();
    assert_eq!(response["status"], "termination_accepted");
    assert!(state.ended && state.exit_confirmed == Some(false) && state.retirement_reason.is_none());
}

fn matches_reference(id: Uuid, name: Option<&str>, reference: &SessionReference) -> bool {
    match Uuid::parse_str(reference.as_str()) {
        Ok(requested) => requested == id,
        Err(_) => name == Some(reference.as_str()),
    }
}

#[cfg(test)]
#[test]
fn guid_reference_never_falls_back_to_a_reused_local_name() {
    let old = Uuid::now_v7();
    let current = Uuid::now_v7();
    let guid = SessionReference::parse(&old.to_string()).unwrap();
    assert!(matches_reference(old, Some("reusable"), &guid));
    assert!(!matches_reference(current, Some(guid.as_str()), &guid));
    assert!(matches_reference(current, Some("reusable"), &SessionReference::parse("reusable").unwrap()));
}

fn quote_command_arg(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn session_command(alias: &str, reference: &SessionReference, retries: u32, flags: &ConnectionFlags, create: bool) -> Result<String> {
    let exe = std::env::current_exe()?;
    let mut parts = vec![
        "&".into(),
        quote_command_arg(&exe.to_string_lossy()),
        if create { "connect".into() } else { "resume".into() },
        quote_command_arg(alias),
        reference.as_str().to_owned(),
        "--retries".into(),
        retries.to_string(),
    ];
    if let Some(address) = &flags.address {
        parts.extend(["--address".into(), quote_command_arg(address)]);
    }
    if flags.stdio {
        parts.push("--stdio".into());
    }
    if let Some(timeout) = flags.timeout {
        parts.extend(["--timeout".into(), format!("{}s", timeout.as_secs())]);
    }
    if let Some(shell) = &flags.shell {
        parts.extend(["--shell".into(), quote_command_arg(shell)]);
    }
    if let Some(cwd) = &flags.cwd {
        parts.extend(["--cwd".into(), quote_command_arg(cwd)]);
    }
    Ok(parts.join(" "))
}

fn print_recovery(id: Uuid, command: &str) {
    eprintln!("[session] Resume GUID: {id}");
    eprintln!("[session] Resume command: {command}");
}

fn print_session_recovery(alias: &str, state: &store::State, retries: u32, flags: &ConnectionFlags) -> Result<()> {
    let pending_name = (state.mode == store::SessionMode::CreatePending)
        .then_some(state.reference.as_deref()).flatten();
    let reference = SessionReference::parse(pending_name.unwrap_or(&state.id.to_string()))?;
    let command = session_command(alias, &reference, retries, flags, pending_name.is_some())?;
    print_recovery(state.id, &command);
    if state.mode == store::SessionMode::CreatePending && pending_name.is_none() {
        eprintln!("[session] Pending creation has no saved name; GUID recovery can only attach an existing host session.");
    }
    Ok(())
}

fn pause_before_retry(attempt: u32, message: &str, cancelled: impl Fn() -> bool) -> Result<bool> {
    let jitter = u16::from_le_bytes(store::random_claim()?[..2].try_into().unwrap()) as u64 % 251;
    let delay = (500u64 * (1u64 << (attempt - 1).min(6))).min(15_000) + jitter;
    eprintln!("[session] {message} in {delay}ms");
    for _ in 0..(delay / 25 + 1) {
        if cancelled() { return Ok(false); }
        thread::sleep(Duration::from_millis(25));
    }
    Ok(true)
}

const ATTACHMENT_LOST_MESSAGE: &str =
    "[session] Attachment lost; this client did not request termination. Remote session state is unknown.";

#[cfg(test)]
#[test]
fn attachment_loss_does_not_claim_remote_survival() {
    assert_eq!(
        ATTACHMENT_LOST_MESSAGE,
        "[session] Attachment lost; this client did not request termination. Remote session state is unknown."
    );
    assert!(!ATTACHMENT_LOST_MESSAGE.contains("was not terminated"));
}

fn run_session(
    root: &Path,
    alias: &str,
    reference: &SessionReference,
    create: bool,
    flags: ConnectionFlags,
) -> Result<u32> {
    let interactive = Console::is_terminal();
    if !flags.stdio && !interactive { arterm::diagnostics::nonblocking(); }
    let config = client_config::load(root)?;
    let target = client_config::target(&config, alias)?.clone();
    let retries = flags.retries.unwrap_or(DEFAULT_RETRIES);
    let timeout = flags.connection_timeout();
    let state_dir = client_config::state_dir(root, &target);
    let (store, state) = match Store::resolve_with_host(&state_dir, &target.target_id, reference, create,
        flags.shell.as_deref(), flags.cwd.as_deref(), &mut |reference| {
            eprintln!("[session] Resolving host reference {} before terminal attachment.{}",
                reference.as_str(), if interactive { " Ctrl+C cancels lookup." } else { "" });
            client_protocol::retry_lookup(retries, || {
                let mut link = connect_link(&config, &target, flags.address.as_deref(), timeout)?;
                client_protocol::resolve_session(&mut link, &target.target_id, reference)
            }, |attempt| {
                pause_before_retry(attempt, &format!("Resolving host session {attempt}/{retries}"), || false)?;
                Ok(())
            })
        }) {
        Ok(resolved) => resolved,
        Err(error) if error.is::<store::RecoveryBlocked>() => {
            let (store, state) = Store::resolve_for_management(&state_dir, &target.target_id, reference)?;
            if let Ok(Some(reason)) = probe_retirement(&config, &target, flags.address.as_deref(), timeout, &state) {
                store.retire(&target.target_id, state.id, reason)?;
                return Err(store::RetiredSession { id: state.id, reason }.into());
            }
            return Err(error);
        }
        Err(error) => return Err(error),
    };
    ensure!(state.broker_binding_is_safe(),
        "saved state has no safe broker binding; reference retained without adopting a broker");
    let store = std::sync::Arc::new(store);
    let service_store = store.clone();
    let id = state.id;
    eprintln!("[session] GUID {id}; Ctrl+] detaches without killing the remote shell.");
    print_session_recovery(alias, &state, retries, &flags)?;
    let mut owner = Owner::start(root, &target.target_id, alias, id, state.reference.clone())?;
    let console = Console::new(flags.stdio)?;
    let snapshot = std::sync::Arc::new(std::sync::Mutex::new(state.clone()));
    let service_snapshot = snapshot.clone();
    let service_config = config.clone();
    let service_target = target.clone();
    let service_address = flags.address.clone();
    let mut engine = Engine::new(state);
    let admission = engine.transfer_admission();
    let files = std::sync::Mutex::new(file_protocol::Files::default());
    owner.set_service(std::sync::Arc::new(move |operation_id, action, check| {
        let state = service_snapshot.lock().unwrap().clone();
        if matches!(action, Operation::Terminate) {
            if let Some(reason) = state.retirement_reason { return Ok(already_retired(state.id, reason)); }
        } else {
            ensure!(!state.ended, "session has ended or its recovery is blocked");
        }
        ensure!(state.mode == store::SessionMode::Attached, "session is not attached");
        check()?;
        if matches!(action, Operation::FileSend { .. } | Operation::FileReceive { .. }) {
            let ticket = admission.ticket().context("file transfer unsupported or attachment unavailable")?;
            let mut files = files.try_lock().map_err(|_| anyhow::anyhow!("another file operation is active"))?;
            let mut link = connect_link(&service_config, &service_target, service_address.as_deref(), timeout)?;
            return file_protocol::transfer(&mut link, &state, &mut files, &ticket, operation_id, action, check);
        }
        ensure!(matches!(action, Operation::Terminate), "invalid management operation");
        check()?;
        let outcome = terminate_remote(&service_config, &service_target, service_address.as_deref(), timeout, &state)?;
        admission.revoke();
        let mut snapshot = service_snapshot.lock().unwrap();
        let response = termination_response(&mut snapshot, outcome)?;
        service_store.save(&snapshot)?;
        Ok(response)
    }));
    let mut terminal = owner.terminal(console);
    let result = (|| -> Result<u32> {
        for attempt in 0..=retries {
            if owner.detached() { return Ok(0); }
            if attempt > 0 {
                owner.set_state("reconnecting");
                terminal.reading(false);
                if !pause_before_retry(attempt, &format!("Reconnecting {attempt}/{retries}; reusing {id}"),
                    || owner.detached())? { return Ok(0); }
            }
            let mut link = match connect_link(&config, &target, flags.address.as_deref(), timeout) {
                Ok(link) => link,
                Err(error) => {
                    eprintln!("[session] Connection failed: {error:#}");
                    print_session_recovery(alias, &engine.state, retries, &flags)?;
                    if error.is::<LoginRequired>() {
                        owner.set_state("auth_required");
                        return Err(error);
                    }
                    continue;
                }
            };
            let end = engine.run(&mut link, &mut terminal, &mut |state| {
                let mut snapshot = snapshot.lock().unwrap();
                let mut updated = state.clone();
                updated.merge_ended(&snapshot)?;
                let creation_confirmed = snapshot.mode == store::SessionMode::CreatePending
                    && updated.mode == store::SessionMode::Attached && !updated.ended;
                store.save(&updated)?;
                *snapshot = updated;
                if creation_confirmed { print_session_recovery(alias, &snapshot, retries, &flags)?; }
                Ok(())
            });
            let end = match end {
                Ok(end) => end,
                Err(error) if error.is::<SessionRejected>() => {
                    if let Ok(Some(reason)) = probe_retirement(&config, &target, flags.address.as_deref(), timeout, &engine.state) {
                        store.retire(&target.target_id, id, reason)?;
                        return Err(store::RetiredSession { id, reason }.into());
                    }
                    return Err(error);
                }
                Err(error) => return Err(error),
            };
            match end {
                End::Detached => {
                    eprintln!("[session] Detached; remote session retained.");
                    return Ok(0);
                }
                End::Exited(code) => {
                    eprintln!("[session] Remote exit code: {code}");
                    return Ok(code);
                }
                End::Disconnected => {
                    owner.set_state("reconnecting");
                    terminal.reading(false);
                    eprintln!("{ATTACHMENT_LOST_MESSAGE}");
                    print_session_recovery(alias, &engine.state, retries, &flags)?;
                }
            }
        }
        bail!("retry limit reached; recovery record retained for {id}")
    })();
    terminal.reading(false);
    drop(terminal);
    if result.is_err() {
        eprintln!("[session] Automatic recovery stopped. The saved GUID does not guarantee the remote session is still alive.");
        print_session_recovery(alias, &engine.state, retries, &flags)?;
    }
    result
}

fn terminate_session(root: &Path, args: &[String]) -> Result<u32> {
    ensure!(args.len() >= 2, "terminate requires MACHINE SESSION");
    let alias = &args[0];
    let reference = SessionReference::parse(&args[1])?;
    let (flags, json) = output_connection_flags(&args[2..])?;
    ensure!(
        flags.retries.is_none(),
        "terminate does not accept --retries"
    );
    let config = client_config::load(root)?;
    let target = client_config::target(&config, alias)?.clone();
    let owners = local_control::discover(root)?.into_iter().filter(|owner| owner.target_id == target.target_id
        && matches_reference(owner.session_id, owner.reference.as_deref(), &reference))
        .collect::<Vec<_>>();
    ensure!(owners.len() <= 1, "multiple local owners match the session");
    if let Some(owner) = owners.first() {
        let response = local_control::request(owner, Operation::Terminate)?;
        println!("{}", client_output::result(&response, alias, reference.as_str(), json)?);
        return Ok(if matches!(response["status"].as_str(), Some("terminated" | "termination_accepted" | "retired" | "already_retired")) { 0 } else { 1 });
    }
    let (store, mut state) = match Store::resolve_for_management(&client_config::state_dir(root, &target), &target.target_id, &reference) {
        Ok(resolved) => resolved,
        Err(error) => {
            if let Some(retired) = error.downcast_ref::<store::RetiredSession>() {
                let response = already_retired(retired.id, retired.reason);
                println!("{}", client_output::result(&response, alias, reference.as_str(), json)?);
                return Ok(0);
            }
            return Err(error);
        }
    };
    let outcome = terminate_remote(&config, &target, flags.address.as_deref(), flags.connection_timeout(), &state)?;
    let response = termination_response(&mut state, outcome)?;
    store.save(&state)?;
    println!("{}", client_output::result(&response, alias, reference.as_str(), json)?);
    Ok(0)
}

fn output_connection_flags(args: &[String]) -> Result<(ConnectionFlags, bool)> {
    let mut json = false;
    let mut options = Vec::new();
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--json" {
            ensure!(!json, "duplicate option: --json");
            json = true;
        } else {
            options.push(args[index].clone());
            if matches!(args[index].as_str(), "--address" | "--retries" | "--timeout") {
                let flag = args[index].clone();
                options.push(value(args, &mut index, &flag)?);
            }
        }
        index += 1;
    }
    Ok((parse_connection_flags(&options, false)?, json))
}

fn doctor(root: &Path, args: &[String]) -> Result<()> {
    let alias = args.first().filter(|arg| !arg.starts_with("--")).cloned();
    let start = usize::from(alias.is_some());
    let flags = parse_connection_flags(&args[start..], false)?;
    ensure!(flags.retries.is_none(), "doctor does not accept --retries");
    ensure!(flags.timeout.is_none() || alias.is_some(), "--timeout requires doctor ALIAS");
    ensure!(
        flags.address.is_none() || alias.is_some(),
        "--address requires doctor ALIAS"
    );
    let config = client_config::load(root)?;
    let target = alias
        .as_deref()
        .map(|alias| {
            client_config::validate_alias(alias)?;
            client_config::target(&config, alias).map(|target| (alias, target))
        })
        .transpose()?;
    if flags.address.is_none() {
        let path = devtunnel_path(&config)?;
        ensure!(
            path.is_file(),
            "configured devtunnel executable is missing: {}",
            path.display()
        );
        ensure_authenticated(path)?;
        println!("Dependency and authentication are ready.");
    }
    if let Some((alias, target)) = target {
        let mut link = connect_link(&config, target, flags.address.as_deref(), flags.connection_timeout())?;
        client_protocol::doctor(&mut link)?;
        println!("Tunnel and host handshake succeeded for {alias}.");
    }
    Ok(())
}

fn local_command(root: &Path, args: &[String]) -> Result<u32> {
    let verb = &args[0];
    let machine = args.get(1).context("command requires MACHINE SESSION")?;
    client_config::validate_alias(machine)?;
    let reference = SessionReference::parse(args.get(2).context("command requires MACHINE SESSION")?)?;
    let mut lines = None;
    let mut command = None;
    let mut file = None;
    let mut wait = false;
    let mut timeout = None;
    let mut json = false;
    let mut command_id = None;
    let mut index = 3;
    while index < args.len() {
        match args[index].as_str() {
            "--json" if !json => json = true,
            "--file" if matches!(verb.as_str(), "send" | "receive") && file.is_none() => {
                file = Some(value(args, &mut index, "--file")?);
            }
            "--command-id" if matches!(verb.as_str(), "read" | "send") && command_id.is_none() => {
                command_id = Some(Uuid::parse_str(&value(args, &mut index, "--command-id")?).context("invalid command ID")?);
            }
            "--lines" if verb == "read" && lines.is_none() => {
                let n: usize = value(args, &mut index, "--lines")?.parse().context("invalid line count")?;
                ensure!((1..=2000).contains(&n), "lines must be 1..=2000");
                lines = Some(n);
            }
            "--command" if verb == "send" && command.is_none() => {
                command = Some(value(args, &mut index, "--command")?);
            }
            "--wait" if verb == "send" && !wait => wait = true,
            "--timeout" if verb == "send" && timeout.is_none() => {
                let text = value(args, &mut index, "--timeout")?;
                let seconds: u64 = text.strip_suffix('s').context("timeout must be whole seconds, e.g. 60s")?
                    .parse().context("invalid timeout")?;
                ensure!(seconds > 0, "timeout must be positive and finite");
                timeout = Some(seconds.checked_mul(1000).context("timeout overflow")?);
            }
            other => bail!("unknown or duplicate option: {other}"),
        }
        index += 1;
    }
    let action = match verb.as_str() {
        "read" => {
            ensure!(command_id.is_none() || lines.is_none(), "--command-id and --lines are mutually exclusive");
            if let Some(command_id) = command_id { Operation::CommandStatus { command_id } }
            else { Operation::Read { lines: lines.unwrap_or(20) } }
        }
        "detach" => Operation::Detach,
        "interrupt" => Operation::Interrupt,
        "send" => {
            ensure!(command.is_some() != file.is_some(), "send requires exactly one of --command and --file");
            if let Some(file) = file {
                ensure!(!wait && timeout.is_none() && command_id.is_none(),
                    "--file cannot be combined with --wait, --timeout or --command-id");
                let path = std::path::absolute(file)?;
                arterm::file_transfer::validate_absolute_path(&path)?;
                Operation::FileSend { path: path.to_str().context("source path is not Unicode")?.into() }
            } else {
                ensure!(!wait || timeout.is_some(), "--wait requires explicit --timeout");
                ensure!(!wait || command.is_some(), "--wait is only valid with --command");
                ensure!(command_id.is_none() || command.is_some(), "--command-id is only valid with --command");
                Operation::Send { command: command.unwrap(), timeout_ms: timeout }
            }
        }
        "receive" => {
            let path = file.context("receive requires --file")?;
            arterm::file_transfer::validate_absolute_path(Path::new(&path))?;
            Operation::FileReceive { path }
        }
        _ => unreachable!(),
    };
    let deadline = timeout.map(|ms| std::time::Instant::now().checked_add(Duration::from_millis(ms))
        .context("timeout exceeds supported clock range")).transpose()?;
    let config = client_config::load(root)?;
    let target = client_config::target(&config, machine)?;
    let matches = local_control::discover(root)?.into_iter().filter(|identity| {
        identity.target_id == target.target_id
            && matches_reference(identity.session_id, identity.reference.as_deref(), &reference)
    }).collect::<Vec<_>>();
    ensure!(matches.len() == 1,
        "no unique active managed local client for {machine} {}; connect explicitly (an older unmanaged client cannot be controlled)",
        reference.as_str());
    let operation_id = if verb == "send" { command_id.unwrap_or_else(Uuid::now_v7) } else { Uuid::now_v7() };
    let action = if let Operation::Send { command, timeout_ms } = action {
        let remaining = deadline.map(|deadline| deadline.saturating_duration_since(std::time::Instant::now()));
        if remaining.is_some_and(|remaining| remaining.is_zero()) {
            let response = serde_json::json!({"status":"timeout","phase":"readiness","command_id":operation_id,"submitted":false,"cancelled":false});
            println!("{}", client_output::result(&response, machine, reference.as_str(), json)?);
            return Ok(124);
        }
        Operation::Send { command, timeout_ms: remaining.map(|remaining| remaining.as_millis().max(1) as u64).or(timeout_ms) }
    } else { action };
    let mut response = local_control::request_with_id(&matches[0], action, operation_id)
        .with_context(|| format!("local request failed; command/operation ID {operation_id}; query rather than resubmit"))?;
    if verb == "send" && response["status"] == "ok" && response["host"]["lookup"] == "unknown" {
        response["status"] = "unknown".into();
        response["error"] = "broker has no command record; the request was not resubmitted".into();
    }
    if wait && (response["status"] == "accepted" || response["status"] == "ok") {
        let remaining = deadline.unwrap().saturating_duration_since(std::time::Instant::now());
        if remaining.is_zero() {
            response = serde_json::json!({"status":"timeout","phase":"completion","command_id":operation_id,"submitted":true,"cancelled":false});
        } else {
            response = local_control::request(&matches[0], Operation::Wait {
                command_id: operation_id, timeout_ms: remaining.as_millis().max(1).try_into().context("timeout overflow")?,
            })?;
        }
    }
    if json {
        println!("{}", serde_json::to_string(&response)?);
    } else if verb == "read" && command_id.is_none() && response["status"] == "ok" {
        if let Some(lines) = response["output"]["lines"].as_array() {
            for line in lines { println!("{}", line.as_str().unwrap_or("")); }
        }
        if response["output"]["truncated"] == true { eprintln!("[read] Output truncated to retained/requested lines."); }
        if response["output"]["replay_gap"] == true { eprintln!("[read] Replay gap; output may be incomplete."); }
        if response["output"]["alternate_screen"] == true { eprintln!("[read] Alternate screen snapshot."); }
    } else {
        println!("{}", client_output::result(&response, machine, reference.as_str(), false)?);
    }
    Ok(client_output::exit_code(&response))
}

fn command(args: &[String]) -> Result<u32> {
    if args == ["--privacy"] {
        println!("{}", deployment::PRIVACY);
        return Ok(0);
    }
    let root = deployment::data_root()?;
    match args.first().map(String::as_str) {
        Some("setup") => {
            setup(&root, &args[1..])?;
            Ok(0)
        }
        Some("login" | "--login") => {
            ensure!(args.len() == 1, "arterm --login / login accepts no extra arguments");
            eprintln!("{}", deployment::PRIVACY_NOTICE);
            let config = client_config::load(&root)?;
            login(devtunnel_path(&config)?)?;
            Ok(0)
        }
        Some("logout") if args.len() == 1 => {
            let config = client_config::load(&root)?;
            ensure!(
                !client_config::any_active_session(&root, &config)?,
                "logout refused while a local session attachment is active"
            );
            run_vendor(devtunnel_path(&config)?, &["user", "logout"])?;
            Ok(0)
        }
        Some("add") => {
            add_target(&root, &args[1..])?;
            Ok(0)
        }
        Some("list") if args.get(1).map(String::as_str) == Some("--client") => {
            ensure!(args.len() == 2 || (args.len() == 3 && args[2] == "--json"), "invalid list --client options");
            let mut connections = Vec::new();
            for identity in local_control::discover(&root)? {
                connections.push(local_control::request(&identity, Operation::Inspect)?);
            }
            println!("{}", client_output::clients(&connections, args.len() == 3)?);
            Ok(0)
        }
        Some("list") if args.get(1).map(String::as_str) == Some("--server") => {
            ensure!(args.len() >= 3, "list --server requires MACHINE");
            let machine = &args[2];
            client_config::validate_alias(machine)?;
            let (flags, json) = output_connection_flags(&args[3..])?;
            ensure!(flags.retries.is_none() && !flags.stdio, "invalid server inventory options");
            let config = client_config::load(&root)?;
            let target = client_config::target(&config, machine)?;
            let mut link = connect_link(&config, target, flags.address.as_deref(), flags.connection_timeout())?;
            let mut inventory = client_protocol::list_sessions(&mut link)?;
            let dir = client_config::state_dir(&root, target);
            let mut labels = std::collections::BTreeMap::new();
            for session in store::saved_sessions(&dir)? {
                if let Some(name) = session.session_name {
                    labels.insert(session.session_id.to_string(), name);
                }
            }
            for session in inventory["sessions"].as_array_mut().context("invalid inventory")? {
                let id = Uuid::parse_str(session["id"].as_str().context("invalid session ID")?)?.to_string();
                session["local_reference"] = labels.get(&id).cloned().map(serde_json::Value::String).unwrap_or(serde_json::Value::Null);
                session["has_local_recovery_record"] = dir.join(format!("{id}.dpapi")).is_file().into();
            }
            inventory["machine"] = machine.clone().into();
            println!("{}", client_output::server(&inventory, json)?);
            Ok(0)
        }
        Some("list") => {
            list_targets(&root, &args[1..])?;
            Ok(0)
        }
        Some("remove") => {
            remove_target(&root, &args[1..])?;
            Ok(0)
        }
        Some("connect" | "resume") => {
            let create = args[0] == "connect";
            let alias = args.get(1).context("connect requires an alias")?;
            client_config::validate_alias(alias)?;
            let reference = args.get(2).filter(|value| !value.starts_with("--"));
            let flags = parse_connection_flags(&args[if reference.is_some() { 3 } else { 2 }..], true)?;
            if let Some(reference) = reference {
                run_session(&root, alias, &SessionReference::parse(reference)?, create, flags)
            } else {
                ensure!(create, "resume requires an existing host GUID or alias");
                let reference = SessionReference::parse(&format!("session-{}", Uuid::now_v7()))?;
                println!("{}", session_command(alias, &reference, flags.retries.unwrap_or(DEFAULT_RETRIES), &flags, true)?);
                Ok(0)
            }
        }
        Some("read" | "detach" | "interrupt" | "send" | "receive") => local_command(&root, args),
        Some("terminate") => terminate_session(&root, &args[1..]),
        Some("doctor") => {
            doctor(&root, &args[1..])?;
            Ok(0)
        }
        Some("--version") | Some("-V") if args.len() == 1 => {
            println!("arterm {}", env!("CARGO_PKG_VERSION"));
            Ok(0)
        }
        Some("--help") | Some("-h") if args.len() == 1 => {
            usage();
            Ok(0)
        }
        None => {
            usage();
            Ok(0)
        }
        _ => {
            usage();
            bail!("invalid command or arguments")
        }
    }
}

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let code = match command(&args) {
        Ok(code) => code as i32,
        Err(error) => {
            eprintln!("[client] {error:#}");
            1
        }
    };
    arterm::diagnostics::drain(Duration::from_millis(250));
    std::process::exit(code);
}
