# arTerm: terminal Windows jarak jauh tetap berjalan setelah koneksi putus

<a id="languages"></a>
<details>
<summary>Languages (16)</summary>

[English](../../README.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Español](README.es.md) | [Português (Brasil)](README.pt-BR.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Italiano](README.it.md) | [Русский](README.ru.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [Bahasa Indonesia](README.id.md) | [हिन्दी](README.hi.md) | [العربية](README.ar.md)

</details>

Saat klien atau VPN terputus, build dan agen tetap berjalan di Windows jarak jauh. Kembali ke shell yang sama.

[Siapkan](#native-setup) · [Buktikan shell yang sama](#native-proof) · [Periksa unduhan dan kepercayaan (Inggris)](../../DEVELOPMENT-INSTALL.md)

<img src="../../.github/arterm-before-after.png" width="700" alt="Konsep: terminal Windows/Dev Box alih-alih aliran seluruh desktop RDP. Setelah jaringan klien terputus atau notebook ditutup, sambungkan kembali Shell 7 yang sama dan masih berjalan: MODE PowerShell tetap dev. Host jarak jauh harus tetap berjalan dan masuk.">

*Ilustrasi alur dengan label Inggris, bukan tangkapan nyata atau bukti pengujian.*

- **Pekerjaan berlanjut tanpa klien.** Build, agen dan perintah berdurasi panjang tetap berjalan pada host Windows jarak jauh saat klien ditutup, VPN putus atau laptop dimulai ulang.
- **Kembali ke pekerjaan, bukan shell kosong.** Hubungkan kembali ke proses shell interaktif yang sama, beserta variabel dan direktori kerjanya.
- **Lihat keluaran selama Anda pergi.** Saat terhubung kembali, keluaran terminal tersimpan diputar ulang; penyimpanannya terbatas, mungkin ada celah dan ini bukan log lengkap.

## Persyaratan dan kepercayaan

arTerm adalah perangkat lunak Windows native yang ditulis dalam Rust. Kedua sisi perlu Windows, akun GitHub yang sama untuk terowongan, dan koneksi keluar yang diizinkan. Host harus tetap berjalan dan pengguna tetap masuk. Mulai ulang, logout, kegagalan, matinya host atau exit di shell jarak jauh mengakhiri proses tanpa pemulihan. Kanal perintah jarak jauh yang disetujui mungkin cukup untuk pekerjaan noninteraktif biasa.

Unduhan x64/ARM64 bertanda tangan pengembangan, bukan produksi dengan kepercayaan publik. Sebelum memasang atau menambah kepercayaan, periksa hanya-baca di PowerShell:

1. Isi `$archive` dengan jalur ZIP yang diunduh. `Get-FileHash -LiteralPath $archive -Algorithm SHA256` menghasilkan `.Hash`: bandingkan dengan ZIP bernama sama dalam `SHA256SUMS` publik rilis itu, bukan daftar hasil ekstraksi. Jika hilang atau berbeda, berhenti; kecocokan bukan bukti keamanan.
2. Setelah ZIP cocok, ekstrak dan buka PowerShell di direktori hasil ekstraksi. `Get-FileHash -LiteralPath .\arTerm-Dev.cer -Algorithm SHA256` menghitung hash sertifikat; `$certificate = [System.Security.Cryptography.X509Certificates.X509Certificate2]::new((Resolve-Path -LiteralPath .\arTerm-Dev.cer).Path)` menyimpan objek sertifikat tanpa menampilkan kolomnya; jalankan `$certificate.Subject; $certificate.Thumbprint` untuk menampilkan nama dan sidik jari. Bandingkan dengan [nilai dipatok/pemeriksaan (Inggris)](../../DEVELOPMENT-INSTALL.md#certificate-identity), bukan hanya nama; jangan impor kepercayaan.
3. Isi `$file` bergantian dengan jalur `arTerm-Client-Setup.exe`, `arTerm-Host-Setup.exe`, `arterm.exe`, `arterm-host.exe`. `Get-AuthenticodeSignature -LiteralPath $file` menampilkan `.Status` dan `.SignerCertificate`. Keempatnya harus `Valid`, sidik jari dan byte sertifikat cocok dengan CER yang dipatok. Selesaikan [pemeriksaan identitas/tanda tangan (Inggris)](../../DEVELOPMENT-INSTALL.md#inspect-extracted-executable-signatures-without-running-them) sebelum menjalankan EXE mana pun.

Kepercayaan manual perlu persetujuan eksplisit terpisah dan izin kebijakan; lalu periksa ulang. Non-`Valid`, penanda tangan tak dikenal atau ketidakcocokan berarti berhenti, bukan melewati peringatan/kontrol. Host perlu Windows Script Host/VBScript yang diizinkan.

Output yang disimpan, riwayat shell biasa, dan berkas pemulihan dapat berisi perintah, jalur, kode, atau rahasia. Lindungi di kedua komputer dan samarkan informasi sensitif sebelum berbagi log, tangkapan layar, atau paket dukungan. Perlindungan kredensial DPAPI tidak berarti semua berkas terenkripsi atau tanpa konten. Pengecualian konten diagnostik kesiapan cakupannya lebih sempit.

<a id="native-setup"></a>

## Penyiapan

[Referensi teknis (Inggris)](../../QUICKSTART.md)

Setelah pemeriksaan lulus dan kebijakan mengizinkan, jalankan arTerm-Host-Setup.exe pada host jarak jauh, lalu buka PowerShell baru:

```powershell
arterm-host setup --name my-devbox
```

Selesaikan login terowongan dan simpan perintah pendaftaran yang dicetak host. Instal arTerm-Client-Setup.exe secara lokal, buka PowerShell baru dengan akun GitHub yang sama, lalu jalankan persis perintah pendaftaran dengan jalur host sebenarnya. Klien sudah diinisialisasi; arterm setup tidak diperlukan. Hubungkan dari komputer lokal dengan nama sesi baru:

```powershell
arterm connect my-devbox MyProof --shell powershell
```

<a id="native-proof"></a>

## Membuktikan shell yang sama

Di PowerShell jarak jauh yang terhubung, jalankan dan catat PID serta direktori:

```powershell
$artermProof = 'kept-on-host'
$artermPid = $PID
$artermCwd = (Get-Location).Path
$PID; $artermProof; (Get-Location).Path
```

Tekan Ctrl+] untuk melepaskan klien. Jika terminal menangkap pintasan itu, tutup hanya tab klien lokal. Biarkan host menyala dan pengguna masuk. Ulangi perintah yang sama secara lokal:

```powershell
arterm connect my-devbox MyProof --shell powershell
```

Di PowerShell jarak jauh setelah tersambung kembali:

```powershell
$PID; $artermProof; (Get-Location).Path
($PID -eq $artermPid) -and ($artermProof -eq 'kept-on-host') -and ((Get-Location).Path -eq $artermCwd)
```

Hasil yang diharapkan: PID yang sama, kept-on-host, direktori yang sama, dan True. Gunakan perintah yang sama untuk melanjutkan di kemudian hari. Akhiri pengujian dengan exit di shell jarak jauh: proses berakhir dan status berjalannya tidak dapat dipulihkan.

## Pemulihan dan pembatalan

Jika gagal, periksa akun dan jaringan dengan arterm doctor my-devbox; gunakan arterm --login untuk memulihkan login klien bila perlu. Jangan hapus catatan pemulihan atau instal ulang host yang berjalan. Pembaruan secara default menolak memutus sesi aktif. Pada host, arterm-host stop --disable mempertahankan keadaan berhenti; arterm-host start memulai kembali tetapi tidak memulihkan shell yang hilang. Penghapusan instalasi mempertahankan data pengguna; lihat panduan untuk mencabut kepercayaan sertifikat.

<a id="batasan"></a>
<a id="kontrol-lokal-lanjutan"></a>

send/read terkelola memerlukan sesi PowerShell/pwsh baru yang didukung dan identitas koneksi lokal tepercaya yang cocok. Integrasi tidak dipasang secara retroaktif pada sesi shell yang sudah ada. Pada cabang pengembangan, validasi input langsung dan peninjauan independen arsip transfer belum selesai; rilis tidak mengesahkan semua kemampuan cabang dan versi tak dikenal tetap belum diverifikasi. TUI sembarang, arsitektur campuran dan Windows Sandbox belum disertifikasi. Membuka blokir berkas otomatis bukan jaminan keamanan. Detail ada dalam panduan Inggris.

[Referensi teknis (Inggris)](../../QUICKSTART.md#automation) · [Periksa unduhan dan kepercayaan (Inggris)](../../DEVELOPMENT-INSTALL.md) · [Signing](../../SIGNING.md) · [English README](../../README.md)
