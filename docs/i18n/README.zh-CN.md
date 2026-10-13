# arTerm：断线后继续工作的 Windows 远程终端

<a id="languages"></a>
<details>
<summary>Languages (16)</summary>

[English](../../README.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Español](README.es.md) | [Português (Brasil)](README.pt-BR.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Italiano](README.it.md) | [Русский](README.ru.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [Bahasa Indonesia](README.id.md) | [हिन्दी](README.hi.md) | [العربية](README.ar.md)

</details>

客户端或 VPN 断线后，让远程 Windows 上的构建和编程代理继续运行，再回到同一个 shell。

[设置](#native-setup) · [验证同一 shell](#native-proof) · [核验下载与信任 (英文)](../../DEVELOPMENT-INSTALL.md)

<img src="../../.github/arterm-before-after.png" width="700" alt="概念图：使用 Windows/Dev Box 终端，而不是完整 RDP 桌面串流。客户端网络中断或合上笔记本后，重新连接同一仍在运行的 Shell 7，PowerShell 的 MODE 仍为 dev。远程主机必须保持运行且已登录。">

*流程示意图；英文标签，不是实际截图或测试证明。*

- **断线，工作继续。** 构建、编程代理和长时间命令留在远程 Windows 主机运行，不因客户端关闭、VPN 断开或笔记本重启而停止。
- **回到原来的工作，不是空白 shell。** 重连同一个交互式 shell 进程，保留变量和工作目录。
- **补看离开期间的输出。** 重连会回放保留的终端输出；保留量有上限，可能存在缺口，不是完整日志。

## 要求与信任

arTerm 是原生 Rust Windows 程序。两端须为 Windows，隧道用同一个 GitHub 账户，并允许出站联网。主机须持续运行且用户保持登录；重启、注销、崩溃、关机或远程 shell 的 exit 会结束进程，不会恢复。普通非交互式任务可能已有足够的获准远程命令通道。

当前 x64/ARM64 下载是开发签名版本，其代码签名证书并非 Windows 默认信任的证书，不是正式生产版本。安装或添加信任前，在 PowerShell 中只读核验：

1. 把 `$archive` 设为实际下载的 ZIP 路径；运行 `Get-FileHash -LiteralPath $archive -Algorithm SHA256`，比较 `.Hash` 与该发布页公开 `SHA256SUMS` 中同名 ZIP 的值，不是解压包内的清单。缺失或不匹配时停止；匹配不代表安全。
2. ZIP 匹配后解压，在解压后的目录中打开 PowerShell，用 `Get-FileHash -LiteralPath .\arTerm-Dev.cer -Algorithm SHA256` 计算公开证书的 `.Hash`，用 `$certificate = [System.Security.Cryptography.X509Certificates.X509Certificate2]::new((Resolve-Path -LiteralPath .\arTerm-Dev.cer).Path)` 保存证书对象，不显示字段；随后运行 `$certificate.Subject; $certificate.Thumbprint` 打印名称和指纹，与 [固定值和完整检查（英文）](../../DEVELOPMENT-INSTALL.md#certificate-identity) 比较，不能只看名称，不导入信任。
3. 把 `$file` 逐一设为 `arTerm-Client-Setup.exe`、`arTerm-Host-Setup.exe`、`arterm.exe`、`arterm-host.exe` 的路径，运行 `Get-AuthenticodeSignature -LiteralPath $file`，查看 `.Status` 和 `.SignerCertificate`。四个文件均须为 `Valid`，签名证书的指纹与字节均匹配固定 CER；先完成 [只读身份/签名检查（英文）](../../DEVELOPMENT-INSTALL.md#inspect-extracted-executable-signatures-without-running-them)，再运行任何 EXE。

手动证书信任须另行明确批准且符合政策，批准后重做检查；非 `Valid`、未知签名者或不匹配必须停止，不能绕过系统警告或组织控制。主机还需政策允许的 Windows Script Host/VBScript。

**保护会话隐私：**保留的终端输出、正常 shell 历史和恢复文件可能包含命令、路径、代码或秘密。两端都应妥善保护，分享日志、截图或支持材料前先脱敏。凭据由 DPAPI 保护，不表示所有文件都加密或不含内容；就绪诊断的有限内容排除不适用于全部会话数据。

<a id="native-setup"></a>

## 设置

[技术参考（英文）](../../QUICKSTART.md)

远程运行 arTerm-Host-Setup.exe，打开新的 PowerShell：

```powershell
arterm-host setup --name my-devbox
```

完成隧道登录，保存主机输出的注册命令。本地运行 arTerm-Client-Setup.exe，打开新的 PowerShell，用同一个 GitHub 账户，原样执行主机打印的注册命令（主机路径以实际输出为准）。安装器已初始化客户端，无需 arterm setup。用新会话名在本地连接：

```powershell
arterm connect my-devbox MyProof --shell powershell
```

<a id="native-proof"></a>

## 验证同一 shell

在远程 PowerShell 中执行，记下 PID 和目录：

```powershell
$artermProof = 'kept-on-host'
$artermPid = $PID
$artermCwd = (Get-Location).Path
$PID; $artermProof; (Get-Location).Path
```

按 Ctrl+] 分离客户端；若快捷键被拦截，只关闭本地客户端标签页。保持主机运行且已登录。在本地重复同一连接命令：

```powershell
arterm connect my-devbox MyProof --shell powershell
```

在重连后的远程 PowerShell 中执行：

```powershell
$PID; $artermProof; (Get-Location).Path
($PID -eq $artermPid) -and ($artermProof -eq 'kept-on-host') -and ((Get-Location).Path -eq $artermCwd)
```

预期：相同 PID、kept-on-host、相同目录和 True。下次用同一连接命令继续工作。测试结束在远程 shell 输入 exit，会结束进程，不能恢复运行状态。

## 故障恢复与撤销

失败先运行 arterm doctor my-devbox，检查两端账户和网络；需要时用 arterm --login 显式恢复客户端登录。不要删除恢复记录或重装运行中的主机。更新默认拒绝中断活动会话。主机上 arterm-host stop --disable 可持续停止，arterm-host start 可重新启动，但不会恢复丢失的 shell。卸载保留用户数据；撤销证书信任见安装指南。

<a id="限制"></a>
<a id="高级本地控制"></a>

托管 send/read 需新建受支持的 PowerShell/pwsh 会话及匹配的受信任本地连接身份；旧会话不补装集成。 当前开发分支的直接输入验证和传输归档独立审查尚未完成；发布包不代表当前分支的全部功能都已通过验证，未知版本仍未验证。任意 TUI、混合架构和 Windows Sandbox 未获认证。文件自动解除阻止不代表安全。详情见英文指南。

[技术参考（英文）](../../QUICKSTART.md#automation) · [核验下载与信任 (英文)](../../DEVELOPMENT-INSTALL.md) · [Signing](../../SIGNING.md) · [English README](../../README.md)
