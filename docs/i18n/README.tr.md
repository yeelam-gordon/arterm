# arTerm: bağlantı kesilince devam eden Windows uzak terminali

<a id="languages"></a>
<details>
<summary>Languages (16)</summary>

[English](../../README.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Español](README.es.md) | [Português (Brasil)](README.pt-BR.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Italiano](README.it.md) | [Русский](README.ru.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [Bahasa Indonesia](README.id.md) | [हिन्दी](README.hi.md) | [العربية](README.ar.md)

</details>

İstemci veya VPN bağlantısı kesilse de derlemeler ve ajanlar uzak Windows’ta çalışır. Aynı kabuğa dönün.

[Kur](#native-setup) · [Aynı kabuğu doğrula](#native-proof) · [İndirmeyi ve güveni doğrula (İngilizce)](../../DEVELOPMENT-INSTALL.md)

<img src="../../.github/arterm-before-after.png" width="700" alt="Kavram: tam RDP masaüstü akışı yerine Windows/Dev Box terminali. İstemci ağı kesilince veya dizüstü kapağı kapanınca aynı yaşayan Shell 7’ye yeniden bağlanın: PowerShell MODE dev olarak kalır. Uzak host çalışmaya ve oturum açık kalmaya devam etmelidir.">

*İngilizce etiketli iş akışı çizimi; gerçek ekran görüntüsü veya test kanıtı değil.*

- **İstemci olmadan da iş sürer.** Derlemeler, ajanlar ve uzun komutlar istemci kapanması, VPN kesintisi veya dizüstü yeniden başlatması sırasında uzak Windows bilgisayarında çalışmayı sürdürür.
- **Boş bir kabuğa değil, işinize dönün.** Aynı etkileşimli kabuk sürecine, değişkenleri ve çalışma dizini korunarak yeniden bağlanın.
- **Siz yokken olanları okuyun.** Yeniden bağlantıda saklanan terminal çıktısı oynatılır; saklama sınırlıdır, boşluklar olabilir ve bu tam bir günlük değildir.

## Gereksinimler ve güven

arTerm, Rust ile yazılmış yerel Windows yazılımıdır. İki uçta Windows, tünellerde aynı GitHub hesabı ve izin verilen giden bağlantılar gerekir. Uzak bilgisayar çalışmalı ve kullanıcı oturumu açık kalmalıdır. Yeniden başlatma, kullanıcı çıkışı, çökme, kapanma veya uzak kabukta exit süreci bitirir; geri getirmez. Sıradan etkileşimsiz işler için onaylı uzak komut kanalınız yeterli olabilir.

x64/ARM64 derlemeleri, Windows’un varsayılan olarak güvenmediği bir geliştirme sertifikasıyla imzalanmıştır; bunlar üretim sürümleri değildir. Kurulum veya güven eklemeden önce PowerShell’de salt okunur denetleyin:

1. `$archive` değişkenini indirilen ZIP’in yoluna ayarlayın. `Get-FileHash -LiteralPath $archive -Algorithm SHA256` çıktısındaki `.Hash` değerini o sürümün kamuya açık `SHA256SUMS` dosyasındaki aynı adlı ZIP ile karşılaştırın, çıkarılan listeyle değil. Değer yoksa veya farklıysa durun; eşleşme güvenlik kanıtı değildir.
2. ZIP eşleşince arşivi çıkarın ve çıkarılan dosyaların dizininde PowerShell’i açın. `Get-FileHash -LiteralPath .\arTerm-Dev.cer -Algorithm SHA256` sertifika hash’ini hesaplar; `$certificate = [System.Security.Cryptography.X509Certificates.X509Certificate2]::new((Resolve-Path -LiteralPath .\arTerm-Dev.cer).Path)` sertifika nesnesini kaydeder, alanları göstermez; adı ve parmak izini görüntülemek için ardından `$certificate.Subject; $certificate.Thumbprint` çalıştırın. [Sabitlenen değerler/denetim (İngilizce)](../../DEVELOPMENT-INSTALL.md#certificate-identity) ile karşılaştırın; yalnız ada bakmayın; bu aşamada sertifikayı güvenilen sertifika deposuna aktarmayın.
3. `$file` değişkenini sırayla `arTerm-Client-Setup.exe`, `arTerm-Host-Setup.exe`, `arterm.exe`, `arterm-host.exe` yollarına ayarlayın. `Get-AuthenticodeSignature -LiteralPath $file` `.Status` ve `.SignerCertificate` gösterir. Dördü de `Valid` olmalı; sertifika parmak izi ve baytları sabitlenen CER ile eşleşmelidir. Herhangi bir EXE çalıştırmadan [tam kimlik/imza denetimini (İngilizce)](../../DEVELOPMENT-INSTALL.md#inspect-extracted-executable-signatures-without-running-them) bitirin.

Elle güven ayrı açık onay ve politika izni gerektirir; sonra yeniden denetleyin. `Valid` değilse, imzalayan bilinmiyorsa veya farklıysa durun; uyarıları/denetimleri aşmayın. Uzak bilgisayar izinli Windows Script Host/VBScript gerektirir.

Saklanan çıktı, normal kabuk geçmişi ve kurtarma dosyalarında komutlar, yollar, kod veya sırlar olabilir. İki bilgisayarda koruyun; günlükleri, görüntüleri ve destek dosyalarını paylaşmadan önce hassas bilgileri gizleyin. Kimlik bilgilerinin DPAPI koruması tüm dosyaların şifreli veya içeriksiz olduğu anlamına gelmez. Hazır olma tanılarının içerik dışlamaları daha dar kapsamlıdır.

<a id="native-setup"></a>

## Kurulum

[Teknik başvuru (İngilizce)](../../QUICKSTART.md)

Denetimler başarılı ve politika izin veriyorsa uzak bilgisayarda arTerm-Host-Setup.exe çalıştırıp yeni PowerShell açın:

```powershell
arterm-host setup --name my-devbox
```

Tünel girişini tamamlayıp sunucunun yazdırdığı kayıt komutunu saklayın. Yerelde arTerm-Client-Setup.exe kurun, yeni PowerShell açın ve aynı GitHub hesabıyla bu kayıt komutunu gerçek sunucu yoluyla aynen çalıştırın. İstemci zaten hazırlanmıştır; arterm setup gerekmez. Yeni oturum adıyla yerelden bağlanın:

```powershell
arterm connect my-devbox MyProof --shell powershell
```

<a id="native-proof"></a>

## Aynı kabuğu doğrulama

Bağlı uzak PowerShell içinde çalıştırıp PID ve dizini not edin:

```powershell
$artermProof = 'kept-on-host'
$artermPid = $PID
$artermCwd = (Get-Location).Path
$PID; $artermProof; (Get-Location).Path
```

Ctrl+] ile yalnızca istemciyi ayırın. Kısayol engelleniyorsa sadece yerel istemci sekmesini kapatın. Uzak bilgisayar ve kullanıcı oturumu açık kalsın. Aynı komutu yerelde tekrarlayın:

```powershell
arterm connect my-devbox MyProof --shell powershell
```

Yeniden bağlanan uzak PowerShell içinde:

```powershell
$PID; $artermProof; (Get-Location).Path
($PID -eq $artermPid) -and ($artermProof -eq 'kept-on-host') -and ((Get-Location).Path -eq $artermCwd)
```

Aynı PID, kept-on-host, aynı dizin ve True beklenir. Daha sonra da aynı bağlantı komutuyla devam edin. Denemeyi bitirmek için uzak kabukta exit yazın: süreç biter ve çalışma durumu geri getirilemez.

## Kurtarma ve geri alma

Hatada arterm doctor my-devbox ile hesapları ve ağı kontrol edin; gerekiyorsa arterm --login ile istemci girişini düzeltin. Kurtarma kayıtlarını silmeyin veya çalışan sunucuyu yeniden kurmayın. Güncellemeler varsayılan olarak etkin oturumları kesmez. Sunucuda arterm-host stop --disable durdurulmuş durumda kalmasını sağlar; arterm-host start yeniden başlatır ama kayıp kabukları geri getirmez. Kaldırma kullanıcı verilerini korur. Sertifika güvenini kaldırmak için kılavuza bakın.

<a id="sınırlar"></a>
<a id="gelişmiş-yerel-kontrol"></a>

Yönetilen send/read yeni desteklenen PowerShell/pwsh oturumu ve bağlantıyla eşleşen güvenilir yerel kimlik gerektirir. Eski kabuklara entegrasyon eklenmez. Geliştirme dalının doğrudan giriş doğrulaması ve bağımsız aktarım arşivi incelemesi henüz tamamlanmadı; yayın dalın tüm yeteneklerini onaylamaz, bilinmeyen sürümler doğrulanmamıştır. Rastgele TUI’ler, karma mimariler ve Windows Sandbox sertifikalı değildir. Dosya engelinin otomatik kaldırılması güvenlik garantisi değildir. Ayrıntılar İngilizce kılavuzdadır.

[Teknik başvuru (İngilizce)](../../QUICKSTART.md#automation) · [İndirmeyi ve güveni doğrula (İngilizce)](../../DEVELOPMENT-INSTALL.md) · [Signing](../../SIGNING.md) · [English README](../../README.md)
