# arTerm: terminale remoto Windows persistente dopo una disconnessione

<a id="languages"></a>
<details>
<summary>Languages (16)</summary>

[English](../../README.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Español](README.es.md) | [Português (Brasil)](README.pt-BR.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Italiano](README.it.md) | [Русский](README.ru.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [Bahasa Indonesia](README.id.md) | [हिन्दी](README.hi.md) | [العربية](README.ar.md)

</details>

Se il client o la VPN si disconnettono, build e agenti continuano sul Windows remoto. Torna alla stessa shell.

[Configura](#native-setup) · [Verifica la stessa shell](#native-proof) · [Verifica download e fiducia (inglese)](../../DEVELOPMENT-INSTALL.md)

<img src="../../.github/arterm-before-after.png" width="700" alt="Concetto: terminale Windows/Dev Box invece dello streaming completo del desktop RDP. Dopo una perdita di rete del client o la chiusura del portatile, riconnetti la stessa Shell 7 ancora attiva: MODE di PowerShell resta dev. L’host remoto deve restare attivo e con sessione aperta.">

*Illustrazione del flusso con etichette inglesi, non schermata reale né prova di test.*

- **Il lavoro continua senza il client.** Build, agenti e comandi lunghi restano attivi sull’host Windows remoto alla chiusura del client, perdita della VPN o riavvio del portatile.
- **Torna al lavoro, non a una shell vuota.** Ricollegati allo stesso processo di shell interattiva, con le sue variabili e directory di lavoro.
- **Consulta quanto accaduto durante l’assenza.** La riconnessione riproduce l’output conservato del terminale; la conservazione è limitata, sono possibili lacune e non è un log completo.

## Requisiti e fiducia

arTerm è software Windows nativo scritto in Rust. Servono Windows a entrambe le estremità, lo stesso account GitHub per i tunnel e connessioni in uscita consentite. L’host deve restare in esecuzione con la sessione Windows aperta. Riavvio, logout, crash, spegnimento o exit nella shell remota terminano il processo senza recupero. Per normali attività non interattive può bastare il canale remoto già approvato.

Le build x64/ARM64 sono firmate con un certificato di sviluppo non riconosciuto come attendibile da Windows per impostazione predefinita; non sono versioni di produzione. Prima di installare o aggiungere fiducia, verifica in sola lettura con PowerShell:

1. Imposta `$archive` sul percorso del ZIP scaricato. `Get-FileHash -LiteralPath $archive -Algorithm SHA256` restituisce `.Hash`: confrontalo con il ZIP omonimo nella `SHA256SUMS` pubblica di quella versione, non nella lista estratta. Se manca o differisce, fermati; la corrispondenza non prova sicurezza.
2. Dopo la corrispondenza del ZIP, estrailo e apri PowerShell nella directory estratta. `Get-FileHash -LiteralPath .\arTerm-Dev.cer -Algorithm SHA256` calcola l’hash del certificato; `$certificate = [System.Security.Cryptography.X509Certificates.X509Certificate2]::new((Resolve-Path -LiteralPath .\arTerm-Dev.cer).Path)` salva l’oggetto del certificato senza mostrare i campi; esegui `$certificate.Subject; $certificate.Thumbprint` per visualizzare nome e impronta. Confronta con [valori fissati e verifica (inglese)](../../DEVELOPMENT-INSTALL.md#certificate-identity), non solo il nome, senza importare fiducia.
3. Imposta `$file`, uno per volta, su `arTerm-Client-Setup.exe`, `arTerm-Host-Setup.exe`, `arterm.exe`, `arterm-host.exe`. `Get-AuthenticodeSignature -LiteralPath $file` mostra `.Status` e `.SignerCertificate`. Tutti e quattro devono essere `Valid`, con impronta e byte del certificato uguali al CER fissato. Completa la [verifica identità/firme (inglese)](../../DEVELOPMENT-INSTALL.md#inspect-extracted-executable-signatures-without-running-them) prima di eseguire qualsiasi EXE.

La fiducia manuale richiede approvazione esplicita separata e permesso della policy; poi verifica di nuovo. Non-`Valid`, firmatario sconosciuto o differenza: fermati, senza aggirare avvisi o controlli. L’host richiede Windows Script Host/VBScript consentito.

Output conservato, normale cronologia della shell e file di recupero possono contenere comandi, percorsi, codice o segreti. Proteggili su entrambe le macchine e oscura i dati sensibili prima di condividere log, immagini o materiali di supporto. DPAPI protegge le credenziali, non significa che ogni file sia cifrato o privo di contenuto. Le esclusioni dei diagnostici di disponibilità sono più limitate.

<a id="native-setup"></a>

## Configurazione

[Riferimento tecnico (inglese)](../../QUICKSTART.md)

Dopo le verifiche e con il permesso della policy, esegui arTerm-Host-Setup.exe sull’host remoto e apri un nuovo PowerShell:

```powershell
arterm-host setup --name my-devbox
```

Completa l’accesso al tunnel e conserva il comando di registrazione stampato dall’host. Installa arTerm-Client-Setup.exe localmente, apri un nuovo PowerShell e usa lo stesso account GitHub. Esegui esattamente il comando di registrazione con il percorso reale stampato. Il client è già inizializzato; arterm setup non serve. Con un nuovo nome di sessione, connettiti localmente:

```powershell
arterm connect my-devbox MyProof --shell powershell
```

<a id="native-proof"></a>

## Verificare la stessa shell

Nella PowerShell remota connessa, esegui e annota PID e cartella:

```powershell
$artermProof = 'kept-on-host'
$artermPid = $PID
$artermCwd = (Get-Location).Path
$PID; $artermProof; (Get-Location).Path
```

Premi Ctrl+] per scollegare solo il client. Se il terminale intercetta la scorciatoia, chiudi soltanto la scheda del client locale. Lascia l’host acceso e l’utente connesso; ripeti lo stesso comando localmente:

```powershell
arterm connect my-devbox MyProof --shell powershell
```

Nella PowerShell remota ritrovata:

```powershell
$PID; $artermProof; (Get-Location).Path
($PID -eq $artermPid) -and ($artermProof -eq 'kept-on-host') -and ((Get-Location).Path -eq $artermCwd)
```

Il risultato atteso è lo stesso PID, kept-on-host, la stessa cartella e True. Riutilizza il comando per continuare il lavoro. Per terminare la prova, scrivi exit nella shell remota: termina il processo e il suo stato non è recuperabile.

## Recupero e annullamento

Se fallisce, usa arterm doctor my-devbox per account e rete e, se necessario, arterm --login per recuperare l’accesso del client. Non cancellare dati di recupero né reinstallare un host attivo. Gli aggiornamenti rifiutano per impostazione predefinita di interrompere sessioni attive. Sull’host arterm-host stop --disable mantiene l’arresto; arterm-host start lo riavvia ma non recupera shell perse. La disinstallazione conserva i dati; la guida spiega come revocare la fiducia del certificato.

<a id="limiti"></a>
<a id="controllo-locale-avanzato"></a>

send/read gestiti richiedono una nuova sessione PowerShell/pwsh supportata e l’identità locale affidabile corrispondente alla connessione. Le shell esistenti non sono adattate. Nel branch di sviluppo, la validazione dell’input diretto e la revisione indipendente degli archivi trasferiti restano da completare; la pubblicazione non qualifica tutte le capacità del branch e le versioni sconosciute restano non verificate. TUI arbitrarie, architetture miste e Windows Sandbox non sono certificati. Lo sblocco automatico dei file non ne garantisce la sicurezza. I dettagli sono nella guida inglese.

[Riferimento tecnico (inglese)](../../QUICKSTART.md#automation) · [Verifica download e fiducia (inglese)](../../DEVELOPMENT-INSTALL.md) · [Signing](../../SIGNING.md) · [English README](../../README.md)
