# arTerm：斷線後仍能繼續執行的 Windows 遠端終端機

<a id="languages"></a>
<details>
<summary>Languages (16)</summary>

[English](../../README.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Español](README.es.md) | [Português (Brasil)](README.pt-BR.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Italiano](README.it.md) | [Русский](README.ru.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [Bahasa Indonesia](README.id.md) | [हिन्दी](README.hi.md) | [العربية](README.ar.md)

</details>

用戶端或 VPN 斷線後，讓遠端 Windows 的建置與程式設計代理繼續執行，再回到同一個 shell。

[設定](#native-setup) · [驗證同一 shell](#native-proof) · [驗證下載與信任 (英文)](../../DEVELOPMENT-INSTALL.md)

<img src="../../.github/arterm-before-after.png" width="700" alt="概念圖：使用 Windows/Dev Box 終端，而不是完整 RDP 桌面串流。用戶端網路中斷或闔上筆電後，重新連接同一仍在執行的 Shell 7，PowerShell 的 MODE 仍為 dev。遠端主機必須保持執行且已登入。">

*工作流程示意圖；英文標籤，不是實際畫面或測試證明。*

- **斷線，工作繼續。** 建置、程式設計代理和長時間指令留在遠端 Windows 主機執行，不因用戶端關閉、VPN 斷線或筆電重新啟動而停止。
- **回到原本的工作，不是空白 shell。** 重新連線到同一互動式 shell 程序，保留變數與工作目錄。
- **補看離開期間的輸出。** 重新連線會回放保留的終端輸出；保留量有上限，可能有缺漏，不是完整日誌。

## 需求與信任

arTerm 是原生 Rust Windows 程式。兩端須為 Windows，通道使用同一 GitHub 帳號，並允許對外連線。主機須持續運作且使用者保持登入；重新啟動、登出、當機、關機或遠端 shell 的 exit 會結束程序，不會復原。一般非互動式工作可能已有足夠的獲准遠端命令管道。

目前 x64/ARM64 下載採開發簽署，其程式碼簽章憑證並非 Windows 預設信任的憑證，不是正式生產版本。安裝或加入信任前，在 PowerShell 唯讀檢查：

1. 將 `$archive` 設為實際下載的 ZIP 路徑；執行 `Get-FileHash -LiteralPath $archive -Algorithm SHA256`，比較 `.Hash` 與該發布頁公開 `SHA256SUMS` 中同名 ZIP 的值，不是解壓縮套件內的清單。缺少或不符就停止；相符不代表安全。
2. ZIP 相符後解壓縮，在解壓縮後的目錄中開啟 PowerShell，用 `Get-FileHash -LiteralPath .\arTerm-Dev.cer -Algorithm SHA256` 計算公開憑證的 `.Hash`，用 `$certificate = [System.Security.Cryptography.X509Certificates.X509Certificate2]::new((Resolve-Path -LiteralPath .\arTerm-Dev.cer).Path)` 儲存憑證物件，不顯示欄位；接著執行 `$certificate.Subject; $certificate.Thumbprint` 顯示名稱與指紋，對照 [固定值與完整檢查（英文）](../../DEVELOPMENT-INSTALL.md#certificate-identity)，不能只看名稱，不匯入信任。
3. 將 `$file` 依序設為 `arTerm-Client-Setup.exe`、`arTerm-Host-Setup.exe`、`arterm.exe`、`arterm-host.exe` 的路徑，執行 `Get-AuthenticodeSignature -LiteralPath $file`，查看 `.Status` 與 `.SignerCertificate`。四個檔案皆須為 `Valid`，簽署憑證的指紋與位元組皆符合固定 CER；先完成 [唯讀身分與簽章檢查（英文）](../../DEVELOPMENT-INSTALL.md#inspect-extracted-executable-signatures-without-running-them)，再執行任何 EXE。

手動憑證信任須另行明確核准且符合政策，核准後重做檢查；非 `Valid`、未知簽署者或不符皆須停止，不繞過系統警告或組織控制。主機還需政策允許的 Windows Script Host/VBScript。

請保護兩端保留的終端機輸出、一般 shell 歷程與復原檔案，可能含指令、路徑、程式碼或機密。分享記錄、截圖或支援資料前先遮蔽敏感內容。DPAPI 保護認證資訊，不表示所有檔案皆加密或不含內容；就緒診斷的內容排除範圍較窄。

<a id="native-setup"></a>

## 設定

[技術參考（英文）](../../QUICKSTART.md)

通過上述檢查且政策允許後，在遠端執行 arTerm-Host-Setup.exe，開啟新的 PowerShell：

```powershell
arterm-host setup --name my-devbox
```

完成通道登入並保留主機印出的用戶端註冊指令。本機安裝 arTerm-Client-Setup.exe，開啟新的 PowerShell，使用同一 GitHub 帳號並原樣執行註冊指令，主機路徑以實際輸出為準。安裝程式已初始化用戶端，不需 arterm setup。使用新的工作階段名稱在本機連線：

```powershell
arterm connect my-devbox MyProof --shell powershell
```

<a id="native-proof"></a>

## 驗證同一 shell

在連線後的遠端 PowerShell 執行，記下 PID 與目錄：

```powershell
$artermProof = 'kept-on-host'
$artermPid = $PID
$artermCwd = (Get-Location).Path
$PID; $artermProof; (Get-Location).Path
```

按 Ctrl+] 分離用戶端；若終端機攔截快捷鍵，只關閉本機用戶端分頁。保持主機運作且已登入，在本機重複相同指令：

```powershell
arterm connect my-devbox MyProof --shell powershell
```

在重新連線的遠端 PowerShell 執行：

```powershell
$PID; $artermProof; (Get-Location).Path
($PID -eq $artermPid) -and ($artermProof -eq 'kept-on-host') -and ((Get-Location).Path -eq $artermCwd)
```

預期看到相同 PID、kept-on-host、相同目錄與 True。下次使用相同指令繼續工作。測試結束時在遠端 shell 輸入 exit；這會結束程序，無法恢復執行狀態。

## 復原與撤銷

失敗時先執行 arterm doctor my-devbox，檢查帳號與網路，必要時用 arterm --login 恢復用戶端驗證。不要刪除復原記錄或重新安裝運作中的主機。更新預設拒絕中斷活動工作階段。主機上的 arterm-host stop --disable 可持續停止，arterm-host start 可再啟動，但不會復原已失去的 shell。解除安裝保留使用者資料；撤銷憑證信任請見安裝指南。

<a id="限制"></a>
<a id="進階本機控制"></a>

受管理的 send/read 需新建受支援的 PowerShell/pwsh 工作階段及相符的受信任本機連線身分；舊工作階段不會補上整合。 目前開發分支的直接輸入驗證及傳輸 ZIP 封裝與解壓縮獨立審查尚未完成；發布套件不代表目前分支的所有功能都已通過驗證，未知版本仍未驗證。任意 TUI、混合架構與 Windows Sandbox 未獲認證。自動解除檔案封鎖不代表安全。詳情見英文指南。

[技術參考（英文）](../../QUICKSTART.md#automation) · [驗證下載與信任 (英文)](../../DEVELOPMENT-INSTALL.md) · [Signing](../../SIGNING.md) · [English README](../../README.md)
