# arTerm：切断後も作業を続けられる Windows リモート端末

<a id="languages"></a>
<details>
<summary>Languages (16)</summary>

[English](../../README.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Español](README.es.md) | [Português (Brasil)](README.pt-BR.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Italiano](README.it.md) | [Русский](README.ru.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [Bahasa Indonesia](README.id.md) | [हिन्दी](README.hi.md) | [العربية](README.ar.md)

</details>

クライアントや VPN が切れても、リモート Windows のビルドやコーディングエージェントを動かし続け、同じシェルへ戻れます。

[設定](#native-setup) · [同じシェルを確認](#native-proof) · [ダウンロードと信頼を確認 (英語)](../../DEVELOPMENT-INSTALL.md)

<img src="../../.github/arterm-before-after.png" width="700" alt="概念図：フル RDP デスクトップ配信の代わりに Windows/Dev Box ターミナルを使用する。クライアントの通信断やノート PC を閉じた後、存続している同じ Shell 7 に再接続すると PowerShell の MODE は dev のまま。リモートホストは稼働し、ログイン状態を維持する必要がある。">

*ワークフローの模式図。ラベルは英語で、実画面やテスト証拠ではありません。*

- **接続が切れても作業を継続。** ビルド、コーディングエージェント、長時間のコマンドはリモート Windows ホストで動き続け、クライアント終了・VPN 切断・ノート PC 再起動に左右されません。
- **空のシェルではなく、元の作業へ。** 同じ対話型シェルのプロセスに再接続し、変数と作業ディレクトリを引き継ぎます。
- **離席中の出力を確認。** 再接続時に保持された端末出力を再生します。保持範囲は有限で欠落もあり、完全なログではありません。

## 要件と信頼

arTerm は Rust 製のネイティブ Windows プログラムです。両端が Windows、同じ GitHub アカウントでのトンネル認証、許可された外向き通信が必要です。ホストは稼働中かつログイン中である必要があります。ホストの再起動・ログオフ・クラッシュ・停止やリモートシェルの exit はプロセスを終了し、復元できません。通常の非対話型ジョブには既存の承認済みリモートコマンド経路で十分な場合があります。

x64/ARM64 配布は開発用署名で、一般に信頼される本番用ビルドではありません。インストールや信頼追加の前に PowerShell で読み取り専用の確認をします。

1. `$archive` にダウンロードした ZIP のパスを設定し、`Get-FileHash -LiteralPath $archive -Algorithm SHA256` の `.Hash` を、そのリリースの公開 `SHA256SUMS` にある同名 ZIP と比較します。展開後の一覧ではありません。値がない・異なる場合は停止します。一致は安全性の証明ではありません。
2. ZIP が一致したら展開し、展開先のディレクトリで PowerShell を開きます。`Get-FileHash -LiteralPath .\arTerm-Dev.cer -Algorithm SHA256` は証明書のハッシュ、`$certificate = [System.Security.Cryptography.X509Certificates.X509Certificate2]::new((Resolve-Path -LiteralPath .\arTerm-Dev.cer).Path)` は証明書オブジェクトを保存し、項目を表示しません。その後 `$certificate.Subject; $certificate.Thumbprint` を実行すると名前と拇印が表示されます。[固定値と完全な確認（英語）](../../DEVELOPMENT-INSTALL.md#certificate-identity) と照合し、名前だけで判断せず、信頼は追加しません。
3. `$file` に `arTerm-Client-Setup.exe`、`arTerm-Host-Setup.exe`、`arterm.exe`、`arterm-host.exe` のパスを一つずつ設定して `Get-AuthenticodeSignature -LiteralPath $file` を実行し、`.Status` と `.SignerCertificate` を確認します。四つすべてが `Valid` で、証明書の指紋・バイトが固定 CER と一致する必要があります。EXE を動かす前に [完全な身元・署名確認（英語）](../../DEVELOPMENT-INSTALL.md#inspect-extracted-executable-signatures-without-running-them) を完了してください。

手動の証明書信頼には別途明示的な承認とポリシーの許可が必要で、承認後も再検査します。非 `Valid`・不明な署名者・不一致は停止し、警告や組織の制御を回避しません。ホストには許可された Windows Script Host/VBScript も必要です。

**セッションデータは非公開に：**保持された端末出力、通常のシェル履歴、復旧ファイルにはコマンド、パス、コード、秘密情報が含まれる可能性があります。両側で保護し、ログ、画像、サポート資料を共有する前に機密情報を除去してください。資格情報の DPAPI 保護は全ファイルの暗号化や内容の不在を意味しません。準備状態の診断に限った内容除外は全セッションデータには適用されません。

<a id="native-setup"></a>

## セットアップ

[技術リファレンス（英語）](../../QUICKSTART.md)

ホストで arTerm-Host-Setup.exe を実行し、新しい PowerShell を開きます：

```powershell
arterm-host setup --name my-devbox
```

トンネル認証を完了し、表示された登録コマンドを保存します。ローカルで arTerm-Client-Setup.exe を実行し、新しい PowerShell を開いて同じ GitHub アカウントを使い、ホストの登録コマンドをそのまま実行します。パスは実際の出力に従います。クライアントの初期化は済んでおり arterm setup は不要です。新しいセッション名でローカルから接続します：

```powershell
arterm connect my-devbox MyProof --shell powershell
```

<a id="native-proof"></a>

## 同じシェルを確認

リモート PowerShell で実行し、PID とディレクトリを控えます：

```powershell
$artermProof = 'kept-on-host'
$artermPid = $PID
$artermCwd = (Get-Location).Path
$PID; $artermProof; (Get-Location).Path
```

Ctrl+] で切り離します。効かなければローカルのクライアントタブだけを閉じます。ホストの稼働とログインを保ち、ローカルで同じコマンドを繰り返します：

```powershell
arterm connect my-devbox MyProof --shell powershell
```

再接続したリモート PowerShell で：

```powershell
$PID; $artermProof; (Get-Location).Path
($PID -eq $artermPid) -and ($artermProof -eq 'kept-on-host') -and ((Get-Location).Path -eq $artermCwd)
```

同じ PID、kept-on-host、同じディレクトリ、True が出れば成功です。次回も同じ接続コマンドで続けます。テストを終えるにはリモートで exit を入力します。プロセスが終了し実行状態は復元できません。

## 復旧と元に戻す方法

失敗時は arterm doctor my-devbox で診断し、両側のアカウントと通信を確認します。必要なら arterm --login でクライアント認証を回復します。復旧記録を削除したり稼働中のホストを再インストールしたりしないでください。更新は通常、実行中のセッションがあると拒否されます。ホストで arterm-host stop --disable を使うと停止状態を維持し、arterm-host start で再開できますが失われたシェルは復活しません。アンインストールはデータを残します。信頼解除はインストールガイドにあります。

<a id="制限"></a>
<a id="高度なローカル制御"></a>

管理された send/read は新しい対応 PowerShell/pwsh セッションと一致する信頼済みローカル接続元が必要です。既存シェルには後付けしません。 開発ブランチでは直接入力の検証と転送アーカイブの独立審査が未完了です。配布版はブランチのすべての機能を認定するものではなく、不明な版は未検証です。任意の TUI、混在アーキテクチャ、Windows Sandbox は認定されていません。ファイルの自動ブロック解除は安全性を保証しません。詳細は英語のガイドにあります。

[技術リファレンス（英語）](../../QUICKSTART.md#automation) · [ダウンロードと信頼を確認 (英語)](../../DEVELOPMENT-INSTALL.md) · [Signing](../../SIGNING.md) · [English README](../../README.md)
