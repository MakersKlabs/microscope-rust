<#
.SYNOPSIS
    Sign a Windows executable with Authenticode.

.DESCRIPTION
    Uses signtool.exe and a base64-encoded PFX certificate provided through
    environment variables (typically GitHub Actions secrets):

        WINDOWS_CERT_PFX_BASE64   base64 of the .pfx file
        WINDOWS_CERT_PASSWORD     password of the .pfx file
        WINDOWS_TIMESTAMP_URL     optional, defaults to DigiCert

    If WINDOWS_CERT_PFX_BASE64 is not set the script exits successfully
    without signing, so unsigned builds still work.

    Alternatively you can sign with Azure Trusted Signing by setting
    AZURE_TRUSTED_SIGNING=1 and configuring the azure-signing dlib; see
    docs/SIGNING.md.

.PARAMETER File
    Path of the file to sign.
#>
param(
    [Parameter(Mandatory = $true)][string]$File
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $File)) {
    throw "File not found: $File"
}

if (-not $env:WINDOWS_CERT_PFX_BASE64) {
    Write-Host "WINDOWS_CERT_PFX_BASE64 not set: skipping code signing."
    exit 0
}

$timestampUrl = if ($env:WINDOWS_TIMESTAMP_URL) { $env:WINDOWS_TIMESTAMP_URL } else { "http://timestamp.digicert.com" }

$pfxPath = Join-Path $env:RUNNER_TEMP "codesign.pfx"
[IO.File]::WriteAllBytes($pfxPath, [Convert]::FromBase64String($env:WINDOWS_CERT_PFX_BASE64))

$signtool = (Get-Command signtool.exe -ErrorAction SilentlyContinue)
if (-not $signtool) {
    $kits = "${env:ProgramFiles(x86)}\Windows Kits\10\bin"
    $signtool = Get-ChildItem -Path $kits -Recurse -Filter signtool.exe -ErrorAction SilentlyContinue |
        Where-Object { $_.FullName -match "x64" } |
        Sort-Object FullName -Descending |
        Select-Object -First 1
}
if (-not $signtool) {
    throw "signtool.exe not found. Install the Windows SDK."
}

Write-Host "Signing $File ..."
& $signtool.FullName sign /f "$pfxPath" /p "$env:WINDOWS_CERT_PASSWORD" `
    /fd sha256 /tr "$timestampUrl" /td sha256 "$File"

& $signtool.FullName verify /pa /v "$File"
Write-Host "Signed and verified: $File"
