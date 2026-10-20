# arTerm privacy and local data

This notice describes this source revision and packages built from it, not
previous releases. It is also available offline with `arterm --privacy` or
`arterm-host --privacy`. arTerm is a client and a host you run; the host is your
remote Windows machine, not a project-operated session storage service.

## Which component handles which data

| Component | Responsibility |
|---|---|
| devtunnel and VS Code tunnel tools | Perform account sign-in and manage their own login tokens, authentication caches and service interactions. |
| arTerm | Writes configuration, session-name mappings, session bookkeeping and local diagnostics. Session identifiers and names locate sessions on your host; they are not login credentials. |
| Windows | Provides the user-bound DPAPI protection that arTerm calls before writing protected recovery records. |
| Shells and programs running in a session | Manage their own history, files and application credentials according to their settings. Their storage is separate from arTerm's recovery records. |

Using a dependency does not mean that arTerm implements its storage or telemetry.
Where arTerm starts a dependency, selects its data directory or captures its
output, those actions are identified below.

## Telemetry and network boundaries

arTerm passes `--disable-telemetry` to every VS Code tunnel invocation it owns,
including tunnel start, GitHub login, account inspection and tunnel status.
Its Code CLI help/version probes use the same telemetry-off option.
This disables the vendor telemetry covered by that CLI option. It does not
change an independently started VS Code instance, existing tunnel processes,
extensions, devtunnel, GitHub, Windows diagnostics, or programs run in a shell.
Stopping or restarting the host ends its running sessions.

This is **not** a promise of no network traffic or no third-party data
collection. Downloads, authentication, tunnel discovery and relay operation
still contact Microsoft and GitHub services. They can process account and
device information, IP addresses, tunnel names/IDs, access credentials and
operational metadata. Terminal commands, input/output and file contents travel
between the client and your host through the tunnel. Do not put secrets or
personal information in tunnel names. arTerm does not add its own usage
analytics uploader; local diagnostics are separate from vendor telemetry.

Vendor behavior and retention are governed by their own documentation:
[VS Code Server telemetry](https://code.visualstudio.com/docs/remote/vscode-server),
[Microsoft privacy](https://privacy.microsoft.com/privacystatement),
[dev tunnels](https://learn.microsoft.com/azure/developer/dev-tunnels/overview),
and [GitHub privacy](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement).
No unsupported devtunnel telemetry-off flag is implied.

## Where data lives

For a fresh installation, the default data root on each Windows machine is
`%LOCALAPPDATA%\arTerm`. `VSTERM_REMOTE_HOME` explicitly overrides the root.

New installed binaries use `%LOCALAPPDATA%\Programs\arTerm\Client` or `Host`.
Windows Apps displays **arTerm Client** / **arTerm Host**.

## Client data

- `client\config.json`: local machine aliases, tunnel names/IDs, absolute host
  executable paths, derived target IDs and the selected devtunnel path.
- `client\sessions\<target-id>`: generated session/request/client IDs, local
  cached session-name mappings and recovery records written by arTerm. Recovery includes
  the initial shell/arguments/working directory, terminal dimensions, broker
  identity, session lifecycle state, sequence/acknowledgement state and
  an outstanding unacknowledged input chunk. arTerm clears that chunk when
  acknowledged; this is not a complete command-history recorder or continuous
  tracking of the shell's working directory. Input and launch settings may
  contain user-provided sensitive information. arTerm uses Windows-provided,
  user-bound encryption (DPAPI) for these records. Ordinary name mappings and
  configuration do not use this encryption. DPAPI does not isolate the records
  from other software running as the same Windows user.
- `client\active` contains local control registrations, including the machine
  alias, session name/ID, process and pipe identity. Local attachment/forwarding
  locks, control registration and readiness
  diagnostics support reconnection and automation. Terminal output and command
  results also exist in running-process memory; explicit redirection or
  diagnostic capture can write them to disk.
- devtunnel authentication is vendor-managed, outside arTerm's protected
  recovery records. arTerm does not override devtunnel's credential-store path;
  its version/platform determines that location and storage protection.

Machine aliases are local labels for configured targets. Session names are
sent to the host, which maintains their mapping to session IDs; clients can
cache those mappings locally. Neither kind of name is a login credential.
The host receives session IDs/names, initial shell/arguments/cwd, connection
bookkeeping and terminal data needed for the session. If you reuse a machine
alias as a tunnel name, type it into a command, or include it in a transferred
file, that value is no longer only local. GitHub account identity used for
authentication is separate and handled by the tunnel tools.

## Host, vendor and other retained data

- `host\setup.json`: arTerm-written tunnel name, selected Code CLI path and
  license acceptance.
- `code-cli`: the directory arTerm selects for the VS Code CLI dependency.
  The dependency manages its metadata, tunnel registration and
  authentication/cache data there. Vendor credentials may also use OS-managed stores;
  arTerm does not guarantee that deleting this directory revokes access.
- `host\requests\<request-id>.dpapi`: arTerm-written, Windows-protected
  creation/recovery reservations, including request/session IDs, session names,
  broker identity, timestamp, initial
  shell/arguments/cwd, dimensions and lifecycle state.
- `host\broker.json`, locks, startup task registrations and bootstrap files:
  process/root/installation information used to manage the local host.
  Running sessions hold terminal replay, commands/results and transfer state
  in memory. Detached sessions continue running; there is no live-session idle
  or age expiry.
- `host\code-tunnel.stdout.log` and `host\code-tunnel.stderr.log`: arTerm
  redirects the dependency's stdout/stderr into these append-only files.
  The dependency generates the output; arTerm controls this file capture.
  Scheduled-host logs and `client\diagnostics` / `host\diagnostics`
  contain operational information. Readiness logs use bounded rotation; this
  does **not** impose that same bound or redaction on vendor logs, shell history,
  installer `/log` output or user-captured diagnostics.
- Shells and programs can retain history, files, caches and credentials under
  their own settings outside the arTerm root. File transfers publish files in
  session temporary destinations and can use staging/archive files. Completed
  transferred files are not automatically erased when a session ends.
  Downloaded dependencies and `installer-cache` also persist.

## Retention, deletion and support

There is no blanket automatic data-expiry period. Confirmed retired client
sessions discard pending input, arguments and cwd, but keep
small identity/lifecycle guards and name mappings to prevent unsafe reuse.
Disconnected or unconfirmed recovery state is deliberately retained.
Host recovery reservations and append-only vendor logs are not covered by a
universal expiry rule.

Session bookkeeping applies to the running host/broker instance. A host reboot
or broker restart ends those shells; saved records cannot restore them or
replay old input into a replacement session. Machine aliases, tunnel references
and executable paths remain useful for future connections. Invalidating session
state does not automatically erase its files, logs or backups.

Uninstall retains recovery data, installer cache and vendor dependencies.
Before deletion, finish/terminate sessions and stop/disable the owned host task;
close clients. Deleting client session records loses local names, pending-input
delivery bookkeeping and stale-session guards. It does not terminate a live host
session; an authorized fresh client can resolve an existing session by its host
name or GUID. Do not delete records as a way to retry uncertain input.
After uninstall, you can remove your selected data root and unwanted retained
files, including any separately configured log/history/transfer locations.
Do not delete unrelated data or another installation's files.
Use vendor sign-out/revocation controls separately for
devtunnel, VS Code tunnel and GitHub. Local deletion does not erase backups,
remote copies, third-party logs, or revoke issued credentials.

For general privacy questions use the
[project issues](https://github.com/yeelam/arterm/issues).
**Do not post personal/sensitive data, tokens, session records, logs or
attachments publicly.** Ask for an appropriate private channel before sharing
such evidence; this project does not list a verified private privacy contact.
