#Requires -Version 5.1
<#
.SYNOPSIS
    Generate a self-signed Windows code-signing certificate for the v0 release fallback.

.DESCRIPTION
    Creates a self-signed code-signing certificate (New-SelfSignedCertificate), exports
    it as a password-protected PFX, and prints the SHA-1 thumbprint. This is the v0
    fallback used while no EV/OV certificate is available.

    The exported PFX is intended to be base64-encoded and stored as the GitHub Actions
    secret WINDOWS_CERTIFICATE, with its password in WINDOWS_CERTIFICATE_PASSWORD. See
    docs/distribution.md for the full secret map and the encoding command.

    IMPORTANT: a self-signed certificate is NOT trusted by SmartScreen. End users will
    see an "unknown publisher" warning. When a real EV/OV certificate is procured, drop
    it into the same GitHub secrets and this script is no longer needed — see the
    "Upgrading to a real certificate" note below.

.PARAMETER Subject
    The certificate subject/publisher name shown in the signature. Defaults to the app
    publisher identity.

.PARAMETER PfxPath
    Output path for the exported PFX file. Defaults to .\shalgalt-omr-codesign.pfx
    (relative to the current directory). Do NOT commit this file.

.PARAMETER Password
    Password protecting the PFX. If omitted, the script prompts for a SecureString.

.PARAMETER ValidYears
    Certificate validity in years. Defaults to 3.

.EXAMPLE
    pwsh scripts/gen-self-signed-cert.ps1
    pwsh scripts/gen-self-signed-cert.ps1 -Subject "CN=Shalgalt OMR" -PfxPath C:\secrets\cs.pfx
#>
[CmdletBinding()]
param(
    [string]$Subject = "CN=Shalgalt OMR (Self-Signed v0)",
    [string]$PfxPath = "shalgalt-omr-codesign.pfx",
    [System.Security.SecureString]$Password,
    [int]$ValidYears = 3
)

$ErrorActionPreference = "Stop"

function Write-Log {
    param([string]$Message)
    Write-Host "[gen-self-signed-cert] $Message"
}

if ($null -eq $Password) {
    $Password = Read-Host -AsSecureString -Prompt "Enter a password to protect the exported PFX"
}

Write-Log "Creating self-signed code-signing certificate"
Write-Log "  Subject : $Subject"
Write-Log "  Validity: $ValidYears year(s)"

# Type=CodeSigningCert restricts the EKU to code signing. Stored in CurrentUser\My so it
# does not require elevation; export immediately and remove from the store if desired.
$cert = New-SelfSignedCertificate `
    -Type CodeSigningCert `
    -Subject $Subject `
    -KeyAlgorithm RSA `
    -KeyLength 4096 `
    -HashAlgorithm SHA256 `
    -CertStoreLocation "Cert:\CurrentUser\My" `
    -NotAfter (Get-Date).AddYears($ValidYears)

Write-Log "Certificate created. Thumbprint: $($cert.Thumbprint)"

Write-Log "Exporting PFX -> $PfxPath"
Export-PfxCertificate `
    -Cert ("Cert:\CurrentUser\My\" + $cert.Thumbprint) `
    -FilePath $PfxPath `
    -Password $Password | Out-Null

Write-Log "Exported PFX: $PfxPath"
Write-Host ""
Write-Log "Thumbprint (SHA-1): $($cert.Thumbprint)"
Write-Host ""
Write-Log "Next steps:"
Write-Log "  1. Base64-encode the PFX for the GitHub Actions secret WINDOWS_CERTIFICATE:"
Write-Log "       [Convert]::ToBase64String([IO.File]::ReadAllBytes('$PfxPath')) | Set-Content cert.b64"
Write-Log "     Then paste the contents of cert.b64 into the WINDOWS_CERTIFICATE secret."
Write-Log "  2. Store the PFX password in the WINDOWS_CERTIFICATE_PASSWORD secret."
Write-Log "  3. Do NOT commit the PFX or the base64 file. Delete them once stored."
Write-Host ""
Write-Log "Upgrading to a real certificate:"
Write-Log "  When an EV/OV code-signing certificate is procured, export it to PFX and"
Write-Log "  replace the WINDOWS_CERTIFICATE / WINDOWS_CERTIFICATE_PASSWORD secrets with"
Write-Log "  the real certificate. No code changes are required — release.yml reads the"
Write-Log "  same secrets. This self-signed script is then obsolete."
