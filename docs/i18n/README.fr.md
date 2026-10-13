# arTerm : terminal Windows distant persistant après déconnexion

<a id="languages"></a>
<details>
<summary>Languages (16)</summary>

[English](../../README.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Español](README.es.md) | [Português (Brasil)](README.pt-BR.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Italiano](README.it.md) | [Русский](README.ru.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [Bahasa Indonesia](README.id.md) | [हिन्दी](README.hi.md) | [العربية](README.ar.md)

</details>

Quand le client ou le VPN se déconnecte, compilations et agents continuent sur Windows distant. Retrouvez le même shell.

[Configurer](#native-setup) · [Vérifier le même shell](#native-proof) · [Vérifier téléchargement et confiance (anglais)](../../DEVELOPMENT-INSTALL.md)

<img src="../../.github/arterm-before-after.png" width="700" alt="Concept : terminal Windows/Dev Box plutôt qu’un flux complet de bureau RDP. Après perte du réseau client ou fermeture du portable, reconnectez le même Shell 7 encore actif : MODE PowerShell reste dev. L’hôte distant doit rester actif avec une session ouverte.">

*Illustration du flux, avec libellés anglais : ni capture réelle ni preuve de test.*

- **Le travail continue sans le client.** Compilations, agents et commandes longues restent actifs sur l’hôte Windows distant après fermeture du client, perte du VPN ou redémarrage du portable.
- **Retrouvez votre travail, pas un shell vide.** Reconnectez-vous au même processus de shell interactif, avec ses variables et son répertoire de travail.
- **Consultez ce qui s’est passé en votre absence.** La reconnexion rejoue la sortie conservée du terminal ; la rétention est bornée, des lacunes sont possibles et ce n’est pas un journal complet.

## Prérequis et confiance

arTerm est un programme Windows natif écrit en Rust. Les deux postes doivent utiliser Windows, le même compte GitHub pour les tunnels et des connexions sortantes autorisées. L’hôte doit rester en marche avec une session Windows ouverte. Redémarrage, fermeture de session, crash, arrêt ou exit dans le shell distant terminent le processus sans restauration. Votre canal distant approuvé peut suffire aux tâches non interactives ordinaires.

Les builds x64/ARM64 sont signés avec un certificat de développement qui n’est pas reconnu par défaut comme fiable par Windows ; ce ne sont pas des versions de production. Avant installation ou confiance, vérifiez en lecture seule dans PowerShell :

1. Affectez à `$archive` le chemin du ZIP téléchargé. `Get-FileHash -LiteralPath $archive -Algorithm SHA256` donne `.Hash` : comparez avec le ZIP de même nom dans la `SHA256SUMS` publique de cette version, pas la liste extraite. Valeur absente ou différente : arrêtez ; une égalité ne prouve pas la sécurité.
2. Après correspondance du ZIP, extrayez-le et ouvrez PowerShell dans le répertoire extrait. `Get-FileHash -LiteralPath .\arTerm-Dev.cer -Algorithm SHA256` calcule le hash du certificat ; `$certificate = [System.Security.Cryptography.X509Certificates.X509Certificate2]::new((Resolve-Path -LiteralPath .\arTerm-Dev.cer).Path)` stocke l’objet du certificat sans afficher les champs ; exécutez `$certificate.Subject; $certificate.Thumbprint` pour afficher le nom et l’empreinte. Comparez aux [valeurs fixées et contrôle (anglais)](../../DEVELOPMENT-INSTALL.md#certificate-identity), pas seulement au nom, sans importer de confiance.
3. Affectez successivement à `$file` `arTerm-Client-Setup.exe`, `arTerm-Host-Setup.exe`, `arterm.exe`, `arterm-host.exe`. `Get-AuthenticodeSignature -LiteralPath $file` montre `.Status` et `.SignerCertificate`. Les quatre doivent être `Valid`, avec empreinte et octets du certificat correspondant au CER fixé. Terminez le [contrôle complet identité/signatures (anglais)](../../DEVELOPMENT-INSTALL.md#inspect-extracted-executable-signatures-without-running-them) avant toute exécution EXE.

La confiance manuelle exige une approbation explicite distincte et l’autorisation de la politique ; revérifiez ensuite. Non-`Valid`, signataire inconnu ou écart : arrêtez, sans contourner avertissements ni contrôles. L’hôte exige Windows Script Host/VBScript autorisé.

**Gardez les données de session privées :** sortie conservée, historique normal du shell et fichiers de récupération peuvent contenir commandes, chemins, code ou secrets. Protégez-les sur les deux machines et masquez les informations sensibles avant de partager journaux, captures ou dossiers de support. La protection DPAPI des identifiants ne signifie pas que tous les fichiers sont chiffrés ou sans contenu. Les exclusions de contenu des diagnostics de disponibilité sont plus limitées.

<a id="native-setup"></a>

## Configuration

[Référence technique (anglais)](../../QUICKSTART.md)

Sur l’hôte, exécutez arTerm-Host-Setup.exe puis ouvrez un nouveau PowerShell :

```powershell
arterm-host setup --name my-devbox
```

Terminez l’authentification du tunnel et conservez la commande d’enregistrement affichée. Installez arTerm-Client-Setup.exe localement, ouvrez un nouveau PowerShell et utilisez le même compte GitHub. Exécutez exactement l’enregistrement fourni par l’hôte, avec son chemin réel. L’installation initialise déjà le client ; arterm setup n’est pas nécessaire. Avec un nouveau nom de session, connectez-vous localement :

```powershell
arterm connect my-devbox MyProof --shell powershell
```

<a id="native-proof"></a>

## Vérifier le même shell

Dans le PowerShell distant, exécutez ces lignes et notez le PID et le répertoire :

```powershell
$artermProof = 'kept-on-host'
$artermPid = $PID
$artermCwd = (Get-Location).Path
$PID; $artermProof; (Get-Location).Path
```

Appuyez sur Ctrl+] pour détacher le client. Si le terminal intercepte le raccourci, fermez seulement l’onglet du client local. Gardez l’hôte en marche et l’utilisateur connecté. Relancez localement la même commande :

```powershell
arterm connect my-devbox MyProof --shell powershell
```

Dans le PowerShell distant retrouvé :

```powershell
$PID; $artermProof; (Get-Location).Path
($PID -eq $artermPid) -and ($artermProof -eq 'kept-on-host') -and ((Get-Location).Path -eq $artermCwd)
```

Vous devez retrouver le même PID, kept-on-host, le même répertoire et True. Réutilisez cette commande pour reprendre votre travail lors des prochaines connexions. Pour terminer le test, saisissez exit dans le shell distant : le processus se termine et son état ne peut plus être repris.

## Récupération et retour arrière

En cas d’échec, lancez arterm doctor my-devbox et vérifiez comptes et réseau ; utilisez arterm --login pour rétablir explicitement l’authentification du client si nécessaire. Ne supprimez pas les données de récupération et ne réinstallez pas un hôte actif. Les mises à jour refusent par défaut d’interrompre les sessions actives. Sur l’hôte, arterm-host stop --disable maintient l’arrêt ; arterm-host start le relance sans ressusciter les shells perdus. La désinstallation conserve les données ; le guide explique comment retirer la confiance du certificat.

<a id="limites"></a>
<a id="contrôle-local-avancé"></a>

Les send/read gérés exigent une nouvelle session PowerShell/pwsh prise en charge et l’identité locale approuvée correspondant à la connexion. Les shells existants ne sont pas adaptés. Dans la branche de développement, la validation de l’entrée directe et la revue indépendante des archives de transfert restent à terminer ; la publication ne qualifie pas toutes les capacités de la branche et les versions inconnues restent non vérifiées. Les TUI arbitraires, architectures mixtes et Windows Sandbox ne sont pas certifiés. Le déblocage automatique des fichiers ne garantit pas leur sécurité. Les détails sont dans le guide anglais.

[Référence technique (anglais)](../../QUICKSTART.md#automation) · [Vérifier téléchargement et confiance (anglais)](../../DEVELOPMENT-INSTALL.md) · [Signing](../../SIGNING.md) · [English README](../../README.md)
