# arTerm: persistentes Windows-Remote-Terminal nach Verbindungsabbruch

<a id="languages"></a>
<details>
<summary>Languages (16)</summary>

[English](../../README.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Español](README.es.md) | [Português (Brasil)](README.pt-BR.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Italiano](README.it.md) | [Русский](README.ru.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [Bahasa Indonesia](README.id.md) | [हिन्दी](README.hi.md) | [العربية](README.ar.md)

</details>

Bei Client- oder VPN-Abbruch laufen Builds und Coding-Agenten auf dem Windows-Host weiter. Kehren Sie zur selben Shell zurück.

[Einrichten](#native-setup) · [Dieselbe Shell prüfen](#native-proof) · [Download und Vertrauen prüfen (Englisch)](../../DEVELOPMENT-INSTALL.md)

<img src="../../.github/arterm-before-after.png" width="700" alt="Drei Aktionen: Arbeit läuft trotz Client-Abbruch; dieselbe Shell mit Variablen und Verzeichnis wieder verbinden; gespeicherte Ausgabe nachlesen. Die Wiedergabe ist begrenzt, Lücken sind möglich. Der Host läuft und der Benutzer bleibt angemeldet.">

*Ablaufillustration mit englischen Beschriftungen, kein echter Screenshot oder Testnachweis.*

- **Die Arbeit läuft ohne den Client weiter.** Builds, Agenten und lange Befehle laufen auf dem Windows-Remote-Host auch bei Client-Ende, VPN-Abbruch oder Notebook-Neustart weiter.
- **Zurück zur Arbeit, nicht in eine leere Shell.** Verbinden Sie sich mit demselben interaktiven Shell-Prozess samt Variablen und Arbeitsverzeichnis.
- **Lesen Sie nach, was während Ihrer Abwesenheit geschah.** Beim Wiederverbinden wird gespeicherte Terminalausgabe wiedergegeben; die Aufbewahrung ist begrenzt, Lücken sind möglich, und es ist kein vollständiges Log.

## Voraussetzungen und Vertrauen

arTerm ist native Windows-Software in Rust. Beide Seiten brauchen Windows, dasselbe GitHub-Konto für die Tunnel und erlaubte ausgehende Verbindungen. Der Host muss laufen und der Benutzer angemeldet bleiben. Host-Neustart, Abmeldung, Absturz, Abschalten oder exit in der Remote-Shell beenden den Prozess ohne Wiederherstellung. Für gewöhnliche nichtinteraktive Aufgaben kann der genehmigte Remote-Befehlskanal bereits reichen.

Die x64/ARM64-Downloads sind entwicklungssigniert, keine öffentlich vertrauenswürdigen Produktionsbuilds. Vor Installation oder Vertrauen in PowerShell nur lesend prüfen:

1. Setzen Sie `$archive` auf den tatsächlichen ZIP-Pfad. `Get-FileHash -LiteralPath $archive -Algorithm SHA256` liefert `.Hash`: mit dem gleichnamigen ZIP in der öffentlichen `SHA256SUMS` dieser Veröffentlichung vergleichen, nicht mit der entpackten Liste. Fehlt der Wert oder weicht er ab, stoppen; ein Treffer beweist keine Sicherheit.
2. Nach ZIP-Abgleich entpacken und PowerShell im entpackten Verzeichnis öffnen. `Get-FileHash -LiteralPath .\arTerm-Dev.cer -Algorithm SHA256` liefert den Zertifikat-Hash; `$certificate = [System.Security.Cryptography.X509Certificates.X509Certificate2]::new((Resolve-Path -LiteralPath .\arTerm-Dev.cer).Path)` speichert das Zertifikatsobjekt, ohne die Felder auszugeben; führen Sie `$certificate.Subject; $certificate.Thumbprint` aus, um Name und Fingerabdruck anzuzeigen. Vergleichen Sie die Werte mit den [festgelegten Werten in der verlinkten Prüfung (Englisch)](../../DEVELOPMENT-INSTALL.md#certificate-identity); prüfen Sie nicht nur den Namen und importieren Sie in diesem Schritt kein Zertifikat in einen vertrauenswürdigen Zertifikatspeicher.
3. Setzen Sie `$file` nacheinander auf `arTerm-Client-Setup.exe`, `arTerm-Host-Setup.exe`, `arterm.exe`, `arterm-host.exe`. `Get-AuthenticodeSignature -LiteralPath $file` liefert `.Status` und `.SignerCertificate`. Alle vier müssen `Valid` sein und Zertifikatfingerabdruck/-bytes der festgelegten CER entsprechen. Vor jeder EXE-Ausführung die [vollständige Identitäts-/Signaturprüfung (Englisch)](../../DEVELOPMENT-INSTALL.md#inspect-extracted-executable-signatures-without-running-them) abschließen.

Manuelles Zertifikatsvertrauen braucht separate ausdrückliche Zustimmung und Richtlinienerlaubnis; danach erneut prüfen. Nicht-`Valid`, unbekannte Signierer oder Abweichungen bedeuten Stopp, keinen Warnungs-/Kontrollbypass. Der Host braucht erlaubtes Windows Script Host/VBScript.

**Sitzungsdaten privat halten:** Gespeicherte Terminalausgabe, normale Shell-Historie und Wiederherstellungsdateien können Befehle, Pfade, Code oder Geheimnisse enthalten. Schützen Sie sie auf beiden Rechnern und entfernen Sie sensible Inhalte vor dem Teilen von Logs, Screenshots oder Supportpaketen. DPAPI schützt Zugangsdaten, nicht automatisch alle Dateien vor Klartext oder Inhalt. Die engeren Inhaltsausschlüsse für Bereitschaftsdiagnosen gelten nicht für sämtliche Sitzungsdaten.

<a id="native-setup"></a>

## Einrichtung

[Technische Referenz (Englisch)](../../QUICKSTART.md)

Führen Sie auf dem Host arTerm-Host-Setup.exe aus und öffnen Sie eine neue PowerShell:

```powershell
arterm-host setup --name my-devbox
```

Schließen Sie die Tunnel-Anmeldung ab und bewahren Sie den Registrierungsbefehl auf. Installieren Sie lokal arTerm-Client-Setup.exe, öffnen Sie eine neue PowerShell und verwenden Sie dasselbe GitHub-Konto. Führen Sie die vom Host ausgegebene Registrierung mit dem tatsächlichen Host-Pfad unverändert aus. Der Client ist bereits initialisiert; arterm setup ist nicht nötig. Verbinden Sie sich lokal mit einem neuen Sitzungsnamen:

```powershell
arterm connect my-devbox MyProof --shell powershell
```

<a id="native-proof"></a>

## Dieselbe Shell prüfen

Führen Sie in der entfernten PowerShell diese Befehle aus und notieren Sie PID und Verzeichnis:

```powershell
$artermProof = 'kept-on-host'
$artermPid = $PID
$artermCwd = (Get-Location).Path
$PID; $artermProof; (Get-Location).Path
```

Drücken Sie Ctrl+], um nur den Client zu trennen. Falls das Terminal den Kurzbefehl abfängt, schließen Sie nur den lokalen Client-Tab. Halten Sie den Host am Laufen und den Benutzer angemeldet. Führen Sie lokal denselben Befehl erneut aus:

```powershell
arterm connect my-devbox MyProof --shell powershell
```

In der wieder verbundenen entfernten PowerShell:

```powershell
$PID; $artermProof; (Get-Location).Path
($PID -eq $artermPid) -and ($artermProof -eq 'kept-on-host') -and ((Get-Location).Path -eq $artermCwd)
```

Erwartet werden dieselbe PID, kept-on-host, dasselbe Verzeichnis und True. Verwenden Sie denselben Befehl auch später zum Weiterarbeiten. Zum Beenden des Tests geben Sie exit in der entfernten Shell ein: Der Prozess endet und sein Laufzeitzustand lässt sich nicht wiederherstellen.

## Wiederherstellung und Rücknahme

Bei Fehlern prüfen Sie mit arterm doctor my-devbox Konten und Netzwerk. Bei Bedarf stellt arterm --login die Client-Anmeldung ausdrücklich wieder her. Löschen Sie keine Wiederherstellungsdaten und installieren Sie einen laufenden Host nicht neu. Updates lehnen das Unterbrechen aktiver Sitzungen standardmäßig ab. Auf dem Host hält arterm-host stop --disable den Host angehalten; arterm-host start startet ihn wieder, stellt aber verlorene Shells nicht wieder her. Die Deinstallation behält Daten. Das Entfernen des Zertifikatsvertrauens beschreibt die Installationsanleitung.

<a id="grenzen"></a>
<a id="erweiterte-lokale-steuerung"></a>

Verwaltete send/read benötigen eine neu erstellte unterstützte PowerShell/pwsh-Sitzung und die passende vertrauenswürdige lokale Verbindungsidentität. Bestehende Shells werden nicht nachgerüstet. Im Entwicklungszweig sind die Validierung der Direkteingabe und die unabhängige Prüfung der Transferarchive noch nicht abgeschlossen; die Veröffentlichung qualifiziert nicht alle Zweigfunktionen, unbekannte Versionen bleiben ungeprüft. Beliebige TUIs, gemischte Architekturen und Windows Sandbox sind nicht zertifiziert. Das automatische Aufheben der Downloadblockierung von Dateien ist keine Sicherheitsfreigabe oder -garantie. Details stehen im englischen Leitfaden.

[Technische Referenz (Englisch)](../../QUICKSTART.md#automation) · [Download und Vertrauen prüfen (Englisch)](../../DEVELOPMENT-INSTALL.md) · [Signing](../../SIGNING.md) · [English README](../../README.md)
