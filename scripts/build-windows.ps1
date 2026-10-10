[CmdletBinding()]
param([switch]$SkipInstall)
$ErrorActionPreference = 'Stop'

function Require-Command([string]$Name, [string]$InstallHint) {
    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "$Name was not found. $InstallHint"
    }
}

Require-Command 'node' 'Install Node.js 20 or newer from https://nodejs.org/.'
Require-Command 'npm' 'Install Node.js 20 or newer from https://nodejs.org/.'
Require-Command 'rustc' 'Install Rust stable with https://rustup.rs/ and reopen PowerShell.'
Require-Command 'cargo' 'Install Rust stable with https://rustup.rs/ and reopen PowerShell.'

Push-Location (Join-Path $PSScriptRoot '..')
try {
    if (-not $SkipInstall) {
        Write-Host 'Installing JavaScript dependencies…' -ForegroundColor Cyan
        npm install
        if ($LASTEXITCODE -ne 0) { throw "npm install failed with exit code $LASTEXITCODE" }
    }
    Write-Host 'Building the Mochi Island Windows installer…' -ForegroundColor Cyan
    npm run tauri:build
    if ($LASTEXITCODE -ne 0) { throw "Tauri build failed with exit code $LASTEXITCODE" }

    $bundle = Join-Path (Get-Location) 'src-tauri\target\release\bundle'
    Write-Host "Build complete. Check installer files in: $bundle" -ForegroundColor Green
    if (Test-Path $bundle) { Get-ChildItem $bundle -Recurse -File | Where-Object { $_.Extension -in '.exe', '.msi' } | Select-Object -ExpandProperty FullName }
}
finally { Pop-Location }
