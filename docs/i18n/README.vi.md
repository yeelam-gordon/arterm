# arTerm: terminal Windows từ xa tiếp tục chạy sau khi mất kết nối

<a id="languages"></a>
<details>
<summary>Languages (16)</summary>

[English](../../README.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Español](README.es.md) | [Português (Brasil)](README.pt-BR.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Italiano](README.it.md) | [Русский](README.ru.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [Bahasa Indonesia](README.id.md) | [हिन्दी](README.hi.md) | [العربية](README.ar.md)

</details>

Khi máy khách hoặc VPN mất kết nối, tác vụ biên dịch và tác nhân vẫn chạy trên Windows từ xa. Trở lại đúng shell đó.

[Thiết lập](#native-setup) · [Kiểm tra cùng shell](#native-proof) · [Kiểm tra bản tải và tin cậy (tiếng Anh)](../../DEVELOPMENT-INSTALL.md)

<img src="../../.github/arterm-before-after.png" width="700" alt="Minh họa: dùng terminal Windows/Dev Box thay vì truyền toàn bộ desktop RDP. Sau khi client mất mạng hoặc đóng nắp máy, kết nối lại cùng Shell 7 vẫn hoạt động: MODE PowerShell vẫn là dev. Host từ xa phải tiếp tục chạy và còn đăng nhập.">

*Hình minh họa quy trình, nhãn tiếng Anh; không phải ảnh thực tế hay bằng chứng kiểm thử.*

- **Công việc tiếp tục khi máy khách ngắt kết nối.** Các tác vụ biên dịch, tác nhân và lệnh chạy lâu vẫn tiếp tục chạy trên máy chủ Windows từ xa khi đóng máy khách, mất VPN hoặc khởi động lại laptop.
- **Trở lại công việc, không phải shell trống.** Nối lại đúng tiến trình shell tương tác, giữ nguyên biến và thư mục làm việc.
- **Xem đầu ra trong lúc bạn vắng mặt.** Khi nối lại, đầu ra terminal được giữ sẽ được phát lại; lượng giữ có giới hạn, có thể thiếu đoạn và không phải nhật ký đầy đủ.

## Yêu cầu và tin cậy

arTerm là phần mềm Windows gốc viết bằng Rust. Hai đầu cần Windows, cùng tài khoản GitHub cho đường hầm và kết nối ra ngoài được phép. Máy chủ phải tiếp tục chạy và người dùng vẫn đăng nhập. Khởi động lại, đăng xuất, sự cố, tắt máy hoặc exit trong shell từ xa sẽ kết thúc tiến trình, không phục hồi. Với việc không tương tác thông thường, kênh lệnh từ xa đã được phê duyệt có thể đủ.

Bản tải x64/ARM64 được ký cho phát triển, không có sự tin cậy công khai cho sản xuất. Trước khi cài hoặc thêm tin cậy, kiểm tra chỉ đọc trong PowerShell:

1. Đặt `$archive` thành đường dẫn ZIP đã tải. `Get-FileHash -LiteralPath $archive -Algorithm SHA256` cho `.Hash`: so với ZIP cùng tên trong `SHA256SUMS` công khai của bản phát hành đó, không phải danh sách đã giải nén. Nếu thiếu hoặc khác, dừng; khớp không chứng minh an toàn.
2. Sau khi ZIP khớp, giải nén và mở PowerShell trong thư mục đã giải nén. `Get-FileHash -LiteralPath .\arTerm-Dev.cer -Algorithm SHA256` tính hash chứng chỉ; `$certificate = [System.Security.Cryptography.X509Certificates.X509Certificate2]::new((Resolve-Path -LiteralPath .\arTerm-Dev.cer).Path)` lưu đối tượng chứng chỉ nhưng không hiển thị các trường; sau đó chạy `$certificate.Subject; $certificate.Thumbprint` để hiển thị tên và dấu vân tay. So với [giá trị cố định/kiểm tra (tiếng Anh)](../../DEVELOPMENT-INSTALL.md#certificate-identity), không chỉ tên; không nhập tin cậy.
3. Đặt `$file` lần lượt thành đường dẫn `arTerm-Client-Setup.exe`, `arTerm-Host-Setup.exe`, `arterm.exe`, `arterm-host.exe`. `Get-AuthenticodeSignature -LiteralPath $file` hiển thị `.Status` và `.SignerCertificate`. Cả bốn phải là `Valid`, dấu vân tay và byte chứng chỉ khớp CER cố định. Hoàn tất [kiểm tra danh tính/chữ ký (tiếng Anh)](../../DEVELOPMENT-INSTALL.md#inspect-extracted-executable-signatures-without-running-them) trước khi chạy EXE nào.

Tin cậy thủ công cần phê duyệt rõ ràng riêng và chính sách cho phép; sau đó kiểm tra lại. Không `Valid`, người ký chưa rõ hoặc không khớp: dừng, không vượt cảnh báo/kiểm soát. Máy chủ cần Windows Script Host/VBScript được phép.

Đầu ra lưu lại, lịch sử shell thông thường và tệp khôi phục có thể chứa lệnh, đường dẫn, mã hoặc bí mật. Bảo vệ trên cả hai máy và che thông tin nhạy cảm trước khi chia sẻ nhật ký, ảnh chụp hay gói hỗ trợ. DPAPI bảo vệ thông tin xác thực không có nghĩa mọi tệp được mã hóa hoặc không có nội dung. Việc loại trừ nội dung trong chẩn đoán sẵn sàng có phạm vi hẹp hơn.

<a id="native-setup"></a>

## Thiết lập

[Tài liệu kỹ thuật (tiếng Anh)](../../QUICKSTART.md)

Sau khi kiểm tra đạt và chính sách cho phép, chạy arTerm-Host-Setup.exe từ xa rồi mở PowerShell mới:

```powershell
arterm-host setup --name my-devbox
```

Hoàn tất đăng nhập đường hầm và giữ lệnh đăng ký do máy chủ in ra. Cài arTerm-Client-Setup.exe trên máy cục bộ, mở PowerShell mới với cùng tài khoản GitHub, rồi chạy đúng lệnh đăng ký và đường dẫn máy chủ thực tế. Máy khách đã được khởi tạo; không cần arterm setup. Kết nối cục bộ bằng tên phiên mới:

```powershell
arterm connect my-devbox MyProof --shell powershell
```

<a id="native-proof"></a>

## Kiểm tra cùng shell

Trong PowerShell từ xa vừa kết nối, chạy và ghi lại PID cùng thư mục:

```powershell
$artermProof = 'kept-on-host'
$artermPid = $PID
$artermCwd = (Get-Location).Path
$PID; $artermProof; (Get-Location).Path
```

Nhấn Ctrl+] để tách máy khách. Nếu terminal chặn phím tắt, chỉ đóng tab máy khách cục bộ. Giữ máy chủ chạy và người dùng đăng nhập. Lặp lại cùng lệnh ở máy cục bộ:

```powershell
arterm connect my-devbox MyProof --shell powershell
```

Trong PowerShell từ xa sau khi kết nối lại:

```powershell
$PID; $artermProof; (Get-Location).Path
($PID -eq $artermPid) -and ($artermProof -eq 'kept-on-host') -and ((Get-Location).Path -eq $artermCwd)
```

Kết quả mong đợi là cùng PID, kept-on-host, cùng thư mục và True. Những lần sau dùng cùng lệnh để tiếp tục. Để kết thúc thử nghiệm, nhập exit trong shell từ xa: tiến trình kết thúc và không thể khôi phục trạng thái chạy.

## Khôi phục và hoàn tác

Nếu lỗi, chạy arterm doctor my-devbox để kiểm tra tài khoản và mạng; dùng arterm --login khôi phục đăng nhập máy khách khi cần. Không xóa bản ghi khôi phục hoặc cài lại máy chủ đang chạy. Cập nhật mặc định từ chối ngắt phiên hoạt động. Trên máy chủ, arterm-host stop --disable giữ trạng thái dừng; arterm-host start khởi chạy lại nhưng không phục hồi shell đã mất. Gỡ cài đặt giữ dữ liệu người dùng; xem hướng dẫn để thu hồi tin cậy chứng chỉ.

<a id="giới-hạn"></a>
<a id="điều-khiển-cục-bộ-nâng-cao"></a>

send/read được quản lý cần phiên PowerShell/pwsh mới được hỗ trợ và danh tính kết nối cục bộ đáng tin cậy khớp nhau. Không bổ sung tích hợp vào shell cũ. Trong nhánh phát triển, việc kiểm chứng nhập trực tiếp và rà soát độc lập quy trình đóng gói, giải nén ZIP khi truyền tệp chưa hoàn tất; bản phát hành không chứng nhận mọi khả năng của nhánh và phiên bản chưa rõ vẫn chưa được kiểm chứng. TUI tùy ý, kiến trúc hỗn hợp và Windows Sandbox chưa được chứng nhận. Tự động bỏ chặn tệp không bảo đảm an toàn. Chi tiết ở hướng dẫn tiếng Anh.

[Tài liệu kỹ thuật (tiếng Anh)](../../QUICKSTART.md#automation) · [Kiểm tra bản tải và tin cậy (tiếng Anh)](../../DEVELOPMENT-INSTALL.md) · [Signing](../../SIGNING.md) · [English README](../../README.md)
