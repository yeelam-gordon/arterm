# arTerm: terminal remoto persistente de Windows tras perder la conexión

<a id="languages"></a>
<details>
<summary>Languages (16)</summary>

[English](../../README.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Español](README.es.md) | [Português (Brasil)](README.pt-BR.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Italiano](README.it.md) | [Русский](README.ru.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [Bahasa Indonesia](README.id.md) | [हिन्दी](README.hi.md) | [العربية](README.ar.md)

</details>

Si el cliente o la VPN se desconectan, las compilaciones y los agentes siguen en Windows remoto. Vuelve al mismo shell.

[Configurar](#native-setup) · [Probar el mismo shell](#native-proof) · [Verificar descarga y confianza (inglés)](../../DEVELOPMENT-INSTALL.md)

<img src="../../.github/arterm-before-after.png" width="700" alt="Tres acciones: el trabajo sigue al desconectar el cliente; vuelve al mismo shell, variables y directorio; consulta la salida retenida. La reproducción es limitada y puede tener huecos. El host sigue funcionando con la sesión abierta.">

*Ilustración del flujo con etiquetas en inglés, no captura real ni prueba de ejecución.*

- **El trabajo sigue aunque el cliente se desconecte.** Compilaciones, agentes y comandos largos siguen en el host Windows remoto al cerrar el cliente, perder la VPN o reiniciar el portátil.
- **Vuelve a tu trabajo, no a un shell vacío.** Reconecta al mismo proceso de shell interactivo, con sus variables y directorio de trabajo.
- **Consulta lo ocurrido durante tu ausencia.** La reconexión reproduce la salida retenida del terminal; la retención es limitada, puede haber huecos y no es un registro completo.

## Requisitos y confianza

arTerm es software nativo de Windows escrito en Rust. Ambos extremos necesitan Windows, la misma cuenta GitHub para los túneles y conexiones salientes permitidas. El host debe seguir funcionando con la sesión Windows abierta. Reinicio, cierre de sesión, fallo o apagado del host, o exit en el shell remoto, terminan el proceso sin recuperación. Para tareas no interactivas puede bastar tu canal remoto ya aprobado.

Las descargas x64/ARM64 tienen firma de desarrollo, no confianza pública de producción. Antes de instalar o añadir confianza, verifica sin modificar nada en PowerShell:

1. Asigna a `$archive` la ruta del ZIP descargado. `Get-FileHash -LiteralPath $archive -Algorithm SHA256` devuelve `.Hash`: compáralo con el ZIP del mismo nombre en `SHA256SUMS` público de esa versión, no con la lista extraída. Si falta o difiere, detente; coincidir no prueba seguridad.
2. Cuando coincida el ZIP, extráelo y abre PowerShell en el directorio extraído. `Get-FileHash -LiteralPath .\arTerm-Dev.cer -Algorithm SHA256` calcula el hash del certificado; `$certificate = [System.Security.Cryptography.X509Certificates.X509Certificate2]::new((Resolve-Path -LiteralPath .\arTerm-Dev.cer).Path)` guarda el objeto del certificado sin mostrar los campos; ejecuta `$certificate.Subject; $certificate.Thumbprint` para imprimir el nombre y la huella. Compara con [valores fijados y comprobación (inglés)](../../DEVELOPMENT-INSTALL.md#certificate-identity), no solo el nombre; no importes confianza.
3. Asigna a `$file`, uno por uno, `arTerm-Client-Setup.exe`, `arTerm-Host-Setup.exe`, `arterm.exe`, `arterm-host.exe`. `Get-AuthenticodeSignature -LiteralPath $file` muestra `.Status` y `.SignerCertificate`. Los cuatro deben ser `Valid` y coincidir en huella y bytes con el CER fijado. Completa la [inspección de identidad y firmas (inglés)](../../DEVELOPMENT-INSTALL.md#inspect-extracted-executable-signatures-without-running-them) antes de ejecutar cualquier EXE.

La confianza manual requiere aprobación explícita separada y permiso de la política; repite la inspección después. Estado no `Valid`, firmante desconocido o discrepancia: detente, no evites avisos ni controles. El host necesita Windows Script Host/VBScript permitido.

**Mantén privados los datos de sesión:** la salida retenida, el historial normal del shell y los archivos de recuperación pueden contener comandos, rutas, código o secretos. Protégelos en ambos equipos y elimina información sensible antes de compartir registros, capturas o paquetes de soporte. DPAPI protege credenciales, no garantiza que todos los archivos estén cifrados o sin contenido. Las exclusiones de contenido de los diagnósticos de disponibilidad son más limitadas.

<a id="native-setup"></a>

## Configuración

[Referencia técnica (inglés)](../../QUICKSTART.md)

En el host ejecuta arTerm-Host-Setup.exe y abre un nuevo PowerShell:

```powershell
arterm-host setup --name my-devbox
```

Completa el inicio de sesión del túnel y guarda el comando de registro mostrado. Instala arTerm-Client-Setup.exe localmente, abre un nuevo PowerShell y usa la misma cuenta de GitHub. Ejecuta exactamente el registro impreso por el host con su ruta real. El instalador inicializa el cliente; no hace falta arterm setup. Con un nombre de sesión nuevo, conecta desde el equipo local:

```powershell
arterm connect my-devbox MyProof --shell powershell
```

<a id="native-proof"></a>

## Comprobar el mismo shell

En el PowerShell remoto, ejecuta y anota el PID y el directorio:

```powershell
$artermProof = 'kept-on-host'
$artermPid = $PID
$artermCwd = (Get-Location).Path
$PID; $artermProof; (Get-Location).Path
```

Pulsa Ctrl+] para separar el cliente. Si el terminal captura el atajo, cierra solo la pestaña del cliente local. Mantén el host encendido y el usuario con la sesión de Windows iniciada. Repite localmente el mismo comando:

```powershell
arterm connect my-devbox MyProof --shell powershell
```

En el PowerShell remoto recuperado:

```powershell
$PID; $artermProof; (Get-Location).Path
($PID -eq $artermPid) -and ($artermProof -eq 'kept-on-host') -and ((Get-Location).Path -eq $artermCwd)
```

El resultado esperado es el mismo PID, kept-on-host, el mismo directorio y True. Usa ese comando para continuar en futuras conexiones. Para terminar la prueba, escribe exit en el shell remoto: finaliza el proceso y su estado ya no se recupera.

## Recuperación y reversión

Si falla, ejecuta arterm doctor my-devbox y revisa cuentas y red; usa arterm --login para recuperar explícitamente la autenticación del cliente si hace falta. No borres datos de recuperación ni reinstales un host activo. Las actualizaciones rechazan por defecto interrumpir sesiones activas. En el host, arterm-host stop --disable mantiene la parada; arterm-host start lo reinicia sin recuperar shells perdidos. La desinstalación conserva datos; la guía explica cómo retirar la confianza del certificado.

<a id="límites"></a>
<a id="control-local-avanzado"></a>

send/read gestionados requieren una sesión PowerShell/pwsh compatible recién creada y la identidad local de conexión de confianza coincidente. No se adapta un shell existente. En la rama de desarrollo siguen pendientes la validación de entrada directa y la revisión independiente de los archivos de transferencia; la publicación no certifica todas las capacidades de la rama y las versiones desconocidas siguen sin verificar. No se certifican TUI arbitrarias, arquitecturas mixtas ni Windows Sandbox. Desbloquear archivos automáticamente no garantiza su seguridad. Consulta los detalles en la guía inglesa.

[Referencia técnica (inglés)](../../QUICKSTART.md#automation) · [Verificar descarga y confianza (inglés)](../../DEVELOPMENT-INSTALL.md) · [Signing](../../SIGNING.md) · [English README](../../README.md)
