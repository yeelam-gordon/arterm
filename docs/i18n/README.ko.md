# arTerm: 연결이 끊겨도 유지되는 Windows 원격 터미널

<a id="languages"></a>
<details>
<summary>Languages (16)</summary>

[English](../../README.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Español](README.es.md) | [Português (Brasil)](README.pt-BR.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Italiano](README.it.md) | [Русский](README.ru.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [Bahasa Indonesia](README.id.md) | [हिन्दी](README.hi.md) | [العربية](README.ar.md)

</details>

클라이언트나 VPN 연결이 끊겨도 원격 Windows의 빌드와 코딩 에이전트를 계속 실행하고 같은 셸로 돌아오세요.

[설정](#native-setup) · [같은 셸 확인](#native-proof) · [다운로드와 신뢰 확인 (영어)](../../DEVELOPMENT-INSTALL.md)

<img src="../../.github/arterm-before-after.png" width="700" alt="개념도: 전체 RDP 데스크톱 스트림 대신 Windows/Dev Box 터미널을 사용합니다. 클라이언트 네트워크 단절이나 노트북 덮개 닫기 후 살아 있는 같은 Shell 7에 다시 연결하면 PowerShell MODE는 dev로 유지됩니다. 원격 호스트는 실행 중이고 로그인되어 있어야 합니다.">

*작업 흐름 설명용 그림입니다. 영어 표시이며 실제 화면이나 테스트 증거가 아닙니다.*

- **연결이 끊겨도 작업은 계속됩니다.** 빌드, 코딩 에이전트와 오래 걸리는 명령은 클라이언트 종료, VPN 단절이나 노트북 재시작 중에도 원격 Windows 호스트에서 계속 실행됩니다.
- **빈 셸이 아닌 원래 작업으로 돌아오세요.** 같은 대화형 셸 프로세스에 다시 연결하고 변수와 작업 디렉터리를 유지합니다.
- **자리를 비운 동안의 출력을 확인하세요.** 다시 연결하면 보관된 터미널 출력을 재생합니다. 보관 범위가 제한되어 누락이 있을 수 있으며 완전한 로그가 아닙니다.

## 요구 사항과 신뢰

arTerm은 Rust로 만든 네이티브 Windows 프로그램입니다. 양쪽 모두 Windows이고 터널은 같은 GitHub 계정으로 인증하며 아웃바운드 연결이 허용되어야 합니다. 호스트는 계속 실행 중이며 로그인 상태여야 합니다. 호스트 재부팅·로그아웃·충돌·종료 또는 원격 셸의 exit로 끝난 프로세스는 복원하지 않습니다. 일반 비대화형 작업은 기존에 승인된 원격 명령 채널만으로 충분할 수 있습니다.

x64/ARM64 다운로드는 개발 서명이며 공개적으로 신뢰되는 운영 빌드가 아닙니다. 설치나 신뢰 추가 전에 PowerShell에서 읽기 전용으로 검사하세요.

1. `$archive`를 다운로드한 ZIP 경로로 설정하고 `Get-FileHash -LiteralPath $archive -Algorithm SHA256`의 `.Hash`를 해당 릴리스의 공개 `SHA256SUMS`에서 같은 이름의 ZIP과 비교하세요. 압축을 푼 목록이 아닙니다. 없거나 다르면 중단하세요. 일치는 안전 증명이 아닙니다.
2. ZIP이 일치하면 압축을 풀고 압축을 푼 디렉터리에서 PowerShell을 여세요. `Get-FileHash -LiteralPath .\arTerm-Dev.cer -Algorithm SHA256`는 인증서 해시, `$certificate = [System.Security.Cryptography.X509Certificates.X509Certificate2]::new((Resolve-Path -LiteralPath .\arTerm-Dev.cer).Path)`는 인증서 객체를 저장하며 필드를 출력하지 않습니다. 그 다음 `$certificate.Subject; $certificate.Thumbprint`를 실행하면 이름과 지문이 출력됩니다. [고정 값과 전체 검사(영어)](../../DEVELOPMENT-INSTALL.md#certificate-identity)와 비교하고 이름만으로 판단하거나 인증서를 신뢰하도록 등록하지 마세요.
3. `$file`을 `arTerm-Client-Setup.exe`, `arTerm-Host-Setup.exe`, `arterm.exe`, `arterm-host.exe` 경로로 하나씩 설정하고 `Get-AuthenticodeSignature -LiteralPath $file`으로 `.Status`와 `.SignerCertificate`를 확인하세요. 네 파일 모두 `Valid`이고 인증서 지문과 바이트가 고정 CER와 일치해야 합니다. EXE를 실행하기 전에 [전체 신원/서명 검사(영어)](../../DEVELOPMENT-INSTALL.md#inspect-extracted-executable-signatures-without-running-them)를 완료하세요.

수동 인증서 신뢰는 별도 명시적 승인과 정책 허용이 필요하며 승인 후 재검사하세요. 비 `Valid`, 알 수 없는 서명자 또는 불일치는 중단 사유이며 경고나 통제를 우회하지 마세요. 호스트에는 허용된 Windows Script Host/VBScript도 필요합니다.

보존 출력, 일반 셸 기록, 복구 파일에는 명령, 경로, 코드 또는 비밀이 있을 수 있습니다. 양쪽에서 보호하고 로그, 캡처, 지원 자료를 공유하기 전 민감한 정보를 가리세요. 자격 증명이 DPAPI로 보호된다고 해서 모든 보관 파일이 암호화되거나 민감한 내용이 없는 것은 아닙니다. 준비 상태 진단의 내용 제외는 더 좁은 범위입니다.

<a id="native-setup"></a>

## 설정

[기술 참조 (영어)](../../QUICKSTART.md)

검사에 통과하고 정책이 허용한 뒤 원격에서 arTerm-Host-Setup.exe를 실행하고 새 PowerShell을 엽니다:

```powershell
arterm-host setup --name my-devbox
```

터널 로그인을 마치고 호스트가 출력한 등록 명령을 보관하세요. 로컬에서 arTerm-Client-Setup.exe를 설치하고 새 PowerShell에서 같은 GitHub 계정으로 정확히 그 등록 명령을 실행합니다. 호스트 경로는 실제 출력값을 사용하세요. 설치 프로그램이 초기화하므로 arterm setup은 필요 없습니다. 새 세션 이름으로 로컬에서 연결합니다:

```powershell
arterm connect my-devbox MyProof --shell powershell
```

<a id="native-proof"></a>

## 같은 셸 확인

연결된 원격 PowerShell에서 실행하고 PID와 폴더를 기록합니다:

```powershell
$artermProof = 'kept-on-host'
$artermPid = $PID
$artermCwd = (Get-Location).Path
$PID; $artermProof; (Get-Location).Path
```

Ctrl+]로 클라이언트를 분리합니다. 단축키가 차단되면 로컬 클라이언트 탭만 닫으세요. 호스트를 실행 및 로그인 상태로 유지하고 로컬에서 같은 명령을 반복합니다:

```powershell
arterm connect my-devbox MyProof --shell powershell
```

다시 연결된 원격 PowerShell에서:

```powershell
$PID; $artermProof; (Get-Location).Path
($PID -eq $artermPid) -and ($artermProof -eq 'kept-on-host') -and ((Get-Location).Path -eq $artermCwd)
```

같은 PID, kept-on-host, 같은 폴더와 True가 나오면 성공입니다. 다음에도 같은 명령으로 작업을 이어갑니다. 테스트를 마치려면 원격 셸에서 exit를 입력하세요. 프로세스가 종료되어 실행 상태를 복원할 수 없습니다.

## 복구 및 되돌리기

실패하면 arterm doctor my-devbox로 계정과 네트워크를 확인하고 필요 시 arterm --login으로 클라이언트 인증을 복구하세요. 복구 기록을 삭제하거나 실행 중인 호스트를 재설치하지 마세요. 업데이트는 기본적으로 활성 세션 중단을 거부합니다. 호스트의 arterm-host stop --disable은 정지 상태를 유지하며 arterm-host start로 다시 시작하지만 잃은 셸은 복원하지 않습니다. 제거 후 사용자 데이터는 남습니다. 인증서 신뢰 철회는 설치 가이드를 참조하세요.

<a id="제한"></a>
<a id="고급-로컬-제어"></a>

관리형 send/read는 새로 만든 지원 PowerShell/pwsh 세션과 일치하는 신뢰된 로컬 연결 주체가 필요합니다. 기존 셸에는 통합을 추가하지 않습니다. 개발 브랜치의 직접 입력 검증과 전송 아카이브 독립 검토는 아직 완료되지 않았습니다. 배포본이 브랜치의 모든 기능을 인증하는 것은 아니며 알 수 없는 버전은 미확인입니다. 임의의 TUI, 혼합 아키텍처와 Windows Sandbox는 인증되지 않았습니다. 파일 자동 차단 해제는 안전 보장이 아닙니다. 자세한 내용은 영어 가이드에 있습니다.

[기술 참조 (영어)](../../QUICKSTART.md#automation) · [다운로드와 신뢰 확인 (영어)](../../DEVELOPMENT-INSTALL.md) · [Signing](../../SIGNING.md) · [English README](../../README.md)
