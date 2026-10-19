# arTerm — Reconnect to the same remote Windows terminal

<a id="languages"></a>
<details>
<summary>Languages (16)</summary>

[English](README.md) | [简体中文](docs/i18n/README.zh-CN.md) | [繁體中文](docs/i18n/README.zh-TW.md) | [日本語](docs/i18n/README.ja.md) | [한국어](docs/i18n/README.ko.md) | [Español](docs/i18n/README.es.md) | [Português (Brasil)](docs/i18n/README.pt-BR.md) | [Français](docs/i18n/README.fr.md) | [Deutsch](docs/i18n/README.de.md) | [Italiano](docs/i18n/README.it.md) | [Русский](docs/i18n/README.ru.md) | [Türkçe](docs/i18n/README.tr.md) | [Tiếng Việt](docs/i18n/README.vi.md) | [Bahasa Indonesia](docs/i18n/README.id.md) | [हिन्दी](docs/i18n/README.hi.md) | [العربية](docs/i18n/README.ar.md)

</details>

Keep builds and coding agents running when your client or VPN disconnects.
Return to the same interactive shell, then catch up on retained output.

<img src=".github/arterm-before-after.png" width="700" alt="Concept: use a Windows/Dev Box terminal instead of a full RDP desktop stream. After client network loss or closing the notebook lid, reconnect to the same surviving Shell 7: PowerShell MODE remains dev. The remote host must stay running and logged in.">

*Workflow illustration, not a live screenshot or test evidence. English labels;
[SVG illustration](.github/arterm-before-after.svg).*

- **Work keeps running:** leave the build or agent on the remote host.
- **Return to the same shell:** keep its process, variables and working directory.
- **Catch up on output:** replay retained output; gaps are possible, so save a
  separate build log when you need a complete record.

## Is it for you?

Use arTerm when you need the same host-owned interactive Windows shell after
client closure, VPN loss or notebook restart. An approved remote-command channel
may already suffice for ordinary noninteractive jobs. arTerm uses native Rust
Windows binaries and GitHub-authenticated tunnels, not a remote desktop.

Both ends need Windows, the same GitHub account and permitted outbound access.
The host must stay **running and logged in**. Host reboot, logoff, crash/shutdown
or remote shell `exit` ends the process; arTerm does not resurrect it.

## Get connected

**Current downloads are development-signed, not publicly trusted production
builds.** Before trust, installation or execution, follow the
[ZIP checksum, pinned certificate and read-only executable signature checks](DEVELOPMENT-INSTALL.md).
A matching hash is not safety approval. Require the pinned signer and `Valid`
Windows trust before running; manual certificate trust needs separate explicit,
policy-permitted approval. Never bypass OS warnings or organizational controls.
Host setup also needs policy-permitted Windows Script Host/VBScript.

Retained output, shell history and recovery files may contain code or secrets.
Protect them on both machines and redact shared support material; credential
protection does not encrypt every retained artifact.

**[Safely set up the host and client → QUICKSTART](QUICKSTART.md)**

The guide owns download selection, prerequisites, role installation, sign-in and
host registration. Once configured, connect locally with a new session name:

```powershell
arterm connect my-devbox MyWork --shell powershell
```

An authorized fresh client can resume the same host session without copying
client recovery files:

```powershell
arterm resume my-devbox MyWork
# Or: arterm resume my-devbox <session-guid>
```

The host owns session aliases. Resume and GUID lookup never create sessions;
`connect` can create an absent alias. Saved aliases stay pinned to their GUID.
The verified host Windows owner/session boundary authorizes access, not a
per-session token. Both peers must support `owner-authorized-resume-v1`.

## Prove that you returned to the same shell

Follow the guide's [complete detach/reconnect proof](QUICKSTART.md#prove-that-you-returned-to-the-same-shell).
Expected: unchanged PID, variable and working directory, plus `True`.
This checks client detachment, not host reboot recovery; it is an expected
result, not a run claimed here.

**Ctrl+] detaches** and leaves the remote shell running. **`exit` inside the
remote shell ends it.** Ordinary `arterm-host stop` allows scheduled restart;
`arterm-host stop --disable` keeps the host stopped. Starting it again does not
recover lost shells. See [failure help and safe stopping](QUICKSTART.md#first-connection-help).

## Details when you need them

<a id="what-you-need"></a>
<a id="released-downloads-versus-development-gates"></a>
<a id="release-and-workflow-scope"></a>
- [Prerequisites and released-download evidence](QUICKSTART.md#what-you-need).
  Published v0.7.1 x64/ARM64 ZIPs do not qualify every current-branch capability.
  Direct-input and independent transfer-archive gates remain open; arbitrary
  TUIs, mixed-architecture operation and Windows Sandbox are not certified here.

<a id="control-an-attached-session"></a>
<a id="agent-neutral-command-access"></a>
- [Advanced local control and file transfer](QUICKSTART.md#automation): newly
  created supported PowerShell/pwsh, matching trusted local owner context and
  client bytes; not native integration certification for every coding agent.
  Read the transfer warning before receiving automatically unblocked files.

<a id="session-and-recovery-details"></a>
<a id="installation-and-troubleshooting-details"></a>
<a id="upgrade-and-storage-compatibility"></a>
- [Diagnosis, safe upgrades, stopping and retained data](QUICKSTART.md#upgrade-without-registering-again).

<a id="build-and-test"></a>
<a id="license-and-signing"></a>
- [Technical/source reference](docs/reference.md) · [Signing policy](SIGNING.md) ·
  [Development trust and its removal](DEVELOPMENT-INSTALL.md) · [MIT license](LICENSE).
