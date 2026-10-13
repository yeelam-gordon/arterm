<div dir="rtl">

# arTerm: طرفية Windows بعيدة مستمرة بعد انقطاع الاتصال

</div>

<a id="languages"></a>
<details>
<summary>Languages (16)</summary>

[English](../../README.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Español](README.es.md) | [Português (Brasil)](README.pt-BR.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Italiano](README.it.md) | [Русский](README.ru.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [Bahasa Indonesia](README.id.md) | [हिन्दी](README.hi.md) | [العربية](README.ar.md)

</details>

<div dir="rtl">

عند انقطاع العميل أو VPN، تستمر عمليات البناء والوكلاء على Windows البعيد. عُد إلى مفسّر الأوامر نفسه.

[الإعداد](#native-setup) · [التحقق من مفسّر الأوامر نفسه](#native-proof) · [التحقق من التنزيل والثقة (بالإنجليزية)](../../DEVELOPMENT-INSTALL.md)

</div>

<img src="../../.github/arterm-before-after.png" width="700" alt="رسم تصوري: طرفية Windows/Dev Box بدل بث سطح مكتب RDP كامل. بعد فقد شبكة العميل أو إغلاق غطاء الحاسوب، أعد الاتصال بنفس Shell 7 التي ما زالت تعمل: تبقى قيمة MODE في PowerShell هي dev. يجب أن يبقى المضيف البعيد يعمل مع جلسة مسجلة الدخول.">

<div dir="rtl">

*رسم لسير العمل بتسميات إنجليزية، لا لقطة فعلية ولا دليل اختبار.*

</div>

<div dir="rtl">

- **يستمر العمل دون العميل.** تبقى عمليات البناء والوكلاء والأوامر الطويلة قيد التشغيل على مضيف Windows البعيد عند إغلاق العميل أو انقطاع VPN أو إعادة تشغيل الحاسوب المحمول.
- **عُد إلى عملك، لا إلى مفسّر أوامر فارغ.** اتصل مجددًا بمفسّر الأوامر التفاعلي نفسه، مع متغيراته ودليل العمل.
- **راجع ما ظهر أثناء غيابك.** يعيد الاتصال عرض مخرجات الطرفية المحتفظ بها؛ الاحتفاظ محدود وقد توجد فجوات، وليس سجلًا كاملًا.

</div>

<div dir="rtl">

## المتطلبات والثقة

arTerm برنامج Windows أصلي مكتوب بلغة Rust. يحتاج الطرفان إلى Windows وحساب GitHub نفسه للنفقين واتصالات صادرة مسموح بها. يجب أن يبقى المضيف يعمل والمستخدم مسجّل الدخول. إعادة تشغيل المضيف أو تسجيل الخروج أو التعطل أو الإيقاف أو exit في مفسّر الأوامر البعيد تنهي العملية دون استعادتها. قد تكفي قناة الأوامر البعيدة المعتمدة للمهام غير التفاعلية المعتادة.

تنزيلات x64/ARM64 موقّعة للتطوير وليست إصدارات إنتاج موثوقة لدى الجمهور. قبل التثبيت أو إضافة الثقة، افحص للقراءة فقط في PowerShell:

1. عيّن <code dir="ltr">$archive</code> لمسار ZIP الذي نزّلته. يعطي <code dir="ltr">Get-FileHash -LiteralPath $archive -Algorithm SHA256</code> قيمة <code dir="ltr">.Hash</code>؛ قارنها بملف ZIP ذي الاسم نفسه في <code dir="ltr">SHA256SUMS</code> المنشور لذلك الإصدار، لا بالقائمة داخل الحزمة المستخرجة. إذا غابت القيمة أو اختلفت فتوقف؛ التطابق ليس دليل سلامة.
2. بعد تطابق ZIP، استخرجه وافتح PowerShell في المجلد المستخرج. يحسب <code dir="ltr">Get-FileHash -LiteralPath .\arTerm-Dev.cer -Algorithm SHA256</code> تجزئة الشهادة؛ <code dir="ltr">$certificate = [System.Security.Cryptography.X509Certificates.X509Certificate2]::new((Resolve-Path -LiteralPath .\arTerm-Dev.cer).Path)</code> يخزّن كائن الشهادة دون عرض الحقول؛ نفّذ <code dir="ltr">$certificate.Subject; $certificate.Thumbprint</code> لعرض الاسم وبصمة الشهادة. قارن مع [القيم المحددة والفحص الكامل (بالإنجليزية)](../../DEVELOPMENT-INSTALL.md#certificate-identity)، لا الاسم وحده، دون استيراد الثقة.
3. عيّن <code dir="ltr">$file</code> بالتتابع لمسارات <code dir="ltr">arTerm-Client-Setup.exe</code>, <code dir="ltr">arTerm-Host-Setup.exe</code>, <code dir="ltr">arterm.exe</code>, <code dir="ltr">arterm-host.exe</code>. يعرض <code dir="ltr">Get-AuthenticodeSignature -LiteralPath $file</code> <code dir="ltr">.Status</code> و<code dir="ltr">.SignerCertificate</code>. يجب أن تكون الأربعة <code dir="ltr">Valid</code> وأن تطابق بصمة الشهادة وبايتاتها CER المحدد. أكمل [فحص الهوية والتوقيعات (بالإنجليزية)](../../DEVELOPMENT-INSTALL.md#inspect-extracted-executable-signatures-without-running-them) قبل تشغيل أي EXE.

الثقة اليدوية تحتاج موافقة صريحة مستقلة وسماح السياسة ثم إعادة الفحص. غير <code dir="ltr">Valid</code> أو موقّع مجهول أو اختلاف يعني التوقف، لا تجاوز التحذيرات أو الضوابط. يحتاج المضيف أيضاً إلى Windows Script Host/VBScript المسموح به.

قد تحتوي المخرجات المحفوظة وسجل مفسّر الأوامر المعتاد وملفات الاسترداد على أوامر ومسارات وشيفرة أو أسرار. احمِها على الجهازين واحجب المعلومات الحساسة قبل مشاركة السجلات أو الصور أو حزم الدعم. حماية بيانات الاعتماد عبر DPAPI لا تعني تشفير كل الملفات أو خلوها من المحتوى. استثناءات المحتوى في تشخيصات الجاهزية أضيق نطاقاً.

</div>

<div dir="rtl">

<a id="native-setup"></a>

## الإعداد

[المرجع التقني بالإنجليزية](../../QUICKSTART.md)

بعد نجاح الفحوص وسماح السياسة، شغّل arTerm-Host-Setup.exe على المضيف البعيد وافتح PowerShell جديداً:

</div>

```powershell
arterm-host setup --name my-devbox
```

<div dir="rtl">

أكمل تسجيل دخول النفق واحتفظ بأمر التسجيل الذي يطبعه المضيف. ثبّت arTerm-Client-Setup.exe محلياً، وافتح PowerShell جديداً باستخدام حساب GitHub نفسه، ثم نفّذ أمر التسجيل كما طُبع وبمسار المضيف الفعلي. العميل مهيّأ بالفعل؛ لا حاجة إلى arterm setup. اتصل محلياً باسم جلسة جديد:

</div>

```powershell
arterm connect my-devbox MyProof --shell powershell
```

<div dir="rtl">

<a id="native-proof"></a>

## التحقق من مفسّر الأوامر نفسه

نفّذ في PowerShell البعيد المتصل، وسجّل PID والمجلد:

</div>

```powershell
$artermProof = 'kept-on-host'
$artermPid = $PID
$artermCwd = (Get-Location).Path
$PID; $artermProof; (Get-Location).Path
```

<div dir="rtl">

اضغط Ctrl+] لفصل العميل فقط. إذا اعترضت الطرفية الاختصار، أغلق علامة تبويب العميل المحلي وحدها. أبقِ المضيف يعمل والمستخدم مسجّل الدخول. كرّر الأمر نفسه محلياً:

</div>

```powershell
arterm connect my-devbox MyProof --shell powershell
```

<div dir="rtl">

في PowerShell البعيد بعد إعادة الاتصال:

</div>

```powershell
$PID; $artermProof; (Get-Location).Path
($PID -eq $artermPid) -and ($artermProof -eq 'kept-on-host') -and ((Get-Location).Path -eq $artermCwd)
```

<div dir="rtl">

المتوقع: PID نفسه، وkept-on-host، والمجلد نفسه، وTrue. استخدم أمر الاتصال نفسه لاحقاً لمتابعة العمل. لإنهاء الاختبار، اكتب exit داخل مفسّر الأوامر البعيد: تنتهي العملية ولا يمكن استعادة حالتها أثناء التشغيل.

</div>

<div dir="rtl">

## الاسترداد والتراجع

عند الفشل، افحص الحسابات والشبكة عبر arterm doctor my-devbox، واستعد دخول العميل باستخدام arterm --login عند الحاجة. لا تحذف سجلات الاسترداد ولا تعِد تثبيت مضيف يعمل. ترفض التحديثات افتراضياً قطع الجلسات النشطة. على المضيف يحافظ arterm-host stop --disable على التوقف؛ ويبدأ arterm-host start التشغيل مجدداً دون استعادة جلسات مفسّر الأوامر المفقودة. إزالة التثبيت تحتفظ ببيانات المستخدم؛ راجع الدليل لإلغاء الثقة بالشهادة.

</div>

<div dir="rtl">

<a id="الحدود"></a>
<a id="التحكم-المحلي-المتقدم"></a>

تتطلب send/read المُدارة جلسة PowerShell/pwsh جديدة مدعومة وهوية اتصال محلية موثوقة مطابقة. لا تُضاف آلية التكامل إلى جلسات مفسّر الأوامر القديمة. المراجعة المستقلة لأرشيفات النقل والتحقق من الإدخال المباشر في فرع التطوير لم يكتملَا؛ الإصدار المنشور لا يؤهل كل قدرات الفرع، والإصدارات المجهولة تبقى غير متحققة. واجهات TUI العشوائية والبنى المختلطة وWindows Sandbox غير معتمدة. رفع الحظر عن الملفات تلقائياً لا يضمن سلامتها. التفاصيل في الدليل الإنجليزي.

[المرجع التقني بالإنجليزية](../../QUICKSTART.md#automation) · [التحقق من التنزيل والثقة (بالإنجليزية)](../../DEVELOPMENT-INSTALL.md) · [Signing](../../SIGNING.md) · [English README](../../README.md)

</div>
