#Requires -Version 5.1
<#
.SYNOPSIS
    Download per-OS runtime native libraries into apps/desktop/resources/ on Windows.

.DESCRIPTION
    Fetches the native runtime libraries the Tauri bundle must ship on Windows:

      - pdfium.dll        from bblanchon/pdfium-binaries (pdfium-win-x64.tgz)
      - opencv_world<NNN>.dll  from the OpenCV 4.10.0 Windows self-extractor

    The OpenCV approach mirrors .github/workflows/ci.yml: the
    opencv-<version>-windows.exe artifact is a 7-Zip self-extractor that unpacks to
    C:\tools\opencv\, and the DLL lives at build\x64\vc16\bin. See
    docs/adr/0001-windows-opencv-strategy.md for the rationale.

    Idempotent: skips work when the target file already exists. Pass -Force to
    re-download regardless.

.PARAMETER Force
    Re-download and re-extract even if the target files already exist.

.PARAMETER OpenCvVersion
    OpenCV version to fetch. Keep in lockstep with the OPENCV_VERSION env var in
    .github/workflows/ci.yml. Defaults to 4.10.0.

.EXAMPLE
    pwsh scripts/fetch-binaries.ps1
    pwsh scripts/fetch-binaries.ps1 -Force
#>
[CmdletBinding()]
param(
    [switch]$Force,
    [string]$OpenCvVersion = "4.10.0"
)

$ErrorActionPreference = "Stop"

function Write-Log {
    param([string]$Message)
    Write-Host "[fetch-binaries] $Message"
}

# --- Configuration ----------------------------------------------------------

# Resolve the repository root from this script's own location so the script works
# regardless of the caller's working directory.
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = (Resolve-Path (Join-Path $ScriptDir "..")).Path
$ResourcesDir = Join-Path $RepoRoot "apps\desktop\resources"

$PdfiumUrl = "https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-win-x64.tgz"
$OpenCvToolsRoot = "C:\tools"
$OpenCvDir = Join-Path $OpenCvToolsRoot "opencv"

# --- pdfium ------------------------------------------------------------------

function Get-Pdfium {
    $dest = Join-Path $ResourcesDir "pdfium.dll"
    if ((Test-Path $dest) -and (-not $Force)) {
        Write-Log "pdfium already present: $dest (use -Force to re-download)"
        return
    }

    $tmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ("pdfium-" + [System.Guid]::NewGuid().ToString())
    New-Item -ItemType Directory -Force -Path $tmpDir | Out-Null
    try {
        $archive = Join-Path $tmpDir "pdfium.tgz"
        Write-Log "Downloading pdfium: $PdfiumUrl"
        Invoke-WebRequest -Uri $PdfiumUrl -OutFile $archive

        Write-Log "Extracting pdfium-win-x64.tgz"
        # `tar` ships with Windows 10+; it understands gzip-compressed tarballs.
        tar -xzf $archive -C $tmpDir

        # Archive layout is bin\pdfium.dll. Resolve defensively in case upstream
        # moves it, by searching for the DLL.
        $extracted = Get-ChildItem -Path $tmpDir -Recurse -Filter "pdfium.dll" | Select-Object -First 1
        if ($null -eq $extracted) {
            throw "pdfium.dll not found inside pdfium-win-x64.tgz"
        }

        Copy-Item $extracted.FullName -Destination $dest -Force
        Write-Log "Installed pdfium -> $dest"
    }
    finally {
        Remove-Item -Recurse -Force $tmpDir -ErrorAction SilentlyContinue
    }
}

# --- OpenCV ------------------------------------------------------------------

function Get-OpenCvDllName {
    # opencv_world<MAJOR><MINOR><PATCH> with dots stripped (4.10.0 -> opencv_world4100).
    $libCode = $OpenCvVersion -replace '\.', ''
    return "opencv_world$libCode.dll"
}

function Install-OpenCvSelfExtractor {
    # Mirrors the ci.yml "Install OpenCV (Windows)" step: the windows .exe artifact is
    # a 7-Zip self-extractor whose `-o` flag is the destination root; it creates
    # C:\tools\opencv\ underneath.
    if ((Test-Path $OpenCvDir) -and (-not $Force)) {
        Write-Log "OpenCV already extracted at $OpenCvDir"
        return
    }

    $url = "https://github.com/opencv/opencv/releases/download/$OpenCvVersion/opencv-$OpenCvVersion-windows.exe"
    $tmpExe = Join-Path ([System.IO.Path]::GetTempPath()) "opencv-$OpenCvVersion-windows.exe"

    Write-Log "Downloading OpenCV $OpenCvVersion self-extractor: $url"
    Invoke-WebRequest -Uri $url -OutFile $tmpExe

    New-Item -ItemType Directory -Force -Path $OpenCvToolsRoot | Out-Null
    Write-Log "Extracting OpenCV into $OpenCvDir (this can take a minute)"
    Start-Process -FilePath $tmpExe -ArgumentList "-o`"$OpenCvToolsRoot`" -y" -Wait -NoNewWindow
    Remove-Item $tmpExe -ErrorAction SilentlyContinue
}

function Get-OpenCv {
    $dllName = Get-OpenCvDllName
    $dest = Join-Path $ResourcesDir $dllName
    if ((Test-Path $dest) -and (-not $Force)) {
        Write-Log "$dllName already present: $dest (use -Force to re-download)"
        return
    }

    Install-OpenCvSelfExtractor

    $src = Join-Path $OpenCvDir "build\x64\vc16\bin\$dllName"
    if (-not (Test-Path $src)) {
        throw "$dllName not found at $src — check the OpenCvVersion ($OpenCvVersion) and extraction."
    }

    Copy-Item $src -Destination $dest -Force
    Write-Log "Installed OpenCV runtime -> $dest"
}

# --- Main --------------------------------------------------------------------

Write-Log "Repository root: $RepoRoot"
New-Item -ItemType Directory -Force -Path $ResourcesDir | Out-Null
Write-Log "Resources directory: $ResourcesDir"

Get-Pdfium
Get-OpenCv

Write-Log "Done."
