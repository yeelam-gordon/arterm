# arTerm privacy and local data

This notice describes this source revision and packages built from it, not
previous releases. It is also available offline with `arterm --privacy` or
`arterm-host --privacy`. arTerm is a client and a host you run; the host is your
remote Windows machine, not a project-operated session storage service.

## Telemetry and network boundaries

arTerm passes `--disable-telemetry` to every VS Code tunnel invocation it owns,
including tunnel start, GitHub login, account inspection and tunnel status.
Its Code CLI help/version probes use the same telemetry-off option.
This disables the vendor telemetry covered by that CLI option. It does not
change an independently started VS Code instance, existing tunnel processes,
extensions, devtunnel, GitHub, Windows diagnostics, or programs run in a shell.
An already-running host must be restarted to pick up this source change;
finish needed sessions first because stopping the host ends them.

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

On each Windows machine, the default data root is `%LOCALAPPDATA%\arTerm`.
If `%LOCALAPPDATA%\VsTerm` already exists it is reused, even when both roots
exist. `VSTERM_REMOTE_HOME` explicitly overrides the root. No automatic move,
merge, deletion or import from `DevBoxRemote` occurs. These legacy names are
intentional compatibility contracts to avoid orphaning existing sessions.

New installed binaries use `%LOCALAPPDATA%\Programs\arTerm\Client` or `Host`.
An existing `%LOCALAPPDATA%\Programs\VsTerm\Client` or `Host` is reused for that
role, including mixed layouts. Compatibility EXE aliases, registry/task
ownership, pipe/protocol identities and encryption domains remain unchanged.
Windows Apps displays **arTerm Client** / **arTerm Host**.

## Client data

- `client\config.json`: local machine aliases, tunnel names/IDs, absolute host
  executable paths, derived target IDs and the selected devtunnel path.
- `client\sessions\<target-id>`: generated session/request/client IDs, local
  session-name mappings and protected recovery records. Recovery includes
  shell/arguments/working directory, terminal dimensions, broker identity,
  resume credentials, sequence/acknowledgement state and potentially pending
  input bytes. This is **more than an ID and alias**, and may contain secrets.
  Recovery records use Windows user-bound DPAPI; public name mappings and
  configuration are not encrypted by arTerm. DPAPI is not protection from
  other software running as that user.
- `client\active` contains local control registrations, including the machine
  alias, session name/ID, process and pipe identity. Local attachment/forwarding
  locks, control registration and readiness
  diagnostics support reconnection and automation. Terminal output and command
  results also exist in running-process memory; explicit redirection or
  diagnostic capture can write them to disk.
- devtunnel authentication is vendor-managed, outside arTerm's protected
  recovery records. arTerm does not override devtunnel's credential-store path;
  its version/platform determines that location and storage protection.

Machine aliases and local session names are local labels, not login/user
identities. They are not fields in arTerm's session-create/attach protocol.
The host receives IDs, credentials, shell/arguments/cwd and terminal data
needed for the session. If you reuse an alias as the tunnel name, type it into
a command, or include it in a transferred file, that value is no longer only
local. GitHub account identity used for authentication is separate.

## Host, vendor and other retained data

- `host\setup.json`: tunnel name, selected Code CLI path and license acceptance.
  `code-cli`: isolated VS Code CLI metadata, tunnel registration and vendor
  authentication/cache data. Vendor credentials may also use OS-managed stores;
  arTerm does not guarantee that deleting this directory revokes access.
- `host\requests\<request-id>.dpapi`: protected creation/recovery reservations,
  including request/session IDs, claim/token, broker identity, timestamp,
  shell/arguments/cwd, dimensions and lifecycle state.
- `host\broker.json`, locks, startup task registrations and bootstrap files:
  process/root/installation information used to manage the local host.
  Running sessions hold terminal replay, commands/results and transfer state
  in memory. Detached sessions continue running; there is no live-session idle
  or age expiry.
- `host\code-tunnel.stdout.log` and `host\code-tunnel.stderr.log` append vendor
  output. Scheduled-host logs and `client\diagnostics` / `host\diagnostics`
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
sessions discard recovery secrets, pending input, arguments and cwd, but keep
small identity/lifecycle guards and name mappings to prevent unsafe reuse.
Disconnected or unconfirmed recovery state is deliberately retained.
Host recovery reservations and append-only vendor logs are not covered by a
universal expiry rule.

Uninstall retains recovery data, installer cache and vendor dependencies.
Before deletion, finish/terminate sessions and stop/disable the owned host task;
close clients. Deleting recovery records can permanently prevent reconnection.
After uninstall, you can remove your selected data root and unwanted retained
files, including any separately configured log/history/transfer locations.
Inspect both roots in mixed layouts; do not delete unrelated data or another
installation's files. Use vendor sign-out/revocation controls separately for
devtunnel, VS Code tunnel and GitHub. Local deletion does not erase backups,
remote copies, third-party logs, or revoke issued credentials.

For general privacy questions use the
[project issues](https://github.com/yeelam/arterm/issues).
**Do not post personal/sensitive data, tokens, session records, logs or
attachments publicly.** Ask for an appropriate private channel before sharing
such evidence; this project does not list a verified private privacy contact.
