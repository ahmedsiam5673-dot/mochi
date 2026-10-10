[CmdletBinding()]
param([switch]$Force)
$ErrorActionPreference = 'Stop'
$hookPath = (Resolve-Path (Join-Path $PSScriptRoot 'mochi-hook.ps1')).Path
$settingsDir = Join-Path $HOME '.claude'
$settingsPath = Join-Path $settingsDir 'settings.json'
if (-not (Test-Path $settingsDir)) { New-Item -ItemType Directory -Path $settingsDir -Force | Out-Null }
if (Test-Path $settingsPath) {
    $stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
    Copy-Item $settingsPath "$settingsPath.mochi-backup-$stamp" -Force
    $text = Get-Content $settingsPath -Raw
    if ([string]::IsNullOrWhiteSpace($text)) { $settings = [pscustomobject]@{} }
    else { $settings = $text | ConvertFrom-Json }
} else { $settings = [pscustomobject]@{} }
if (-not ($settings.PSObject.Properties.Name -contains 'hooks') -or $null -eq $settings.hooks) {
    $settings | Add-Member -NotePropertyName hooks -NotePropertyValue ([pscustomobject]@{}) -Force
}
$command = 'powershell.exe -NoProfile -ExecutionPolicy Bypass -File "' + $hookPath + '"'
$events = @('PreToolUse','PostToolUse','Notification','Stop','SessionStart')
foreach ($eventName in $events) {
    $existing = @()
    if ($settings.hooks.PSObject.Properties.Name -contains $eventName -and $null -ne $settings.hooks.$eventName) {
        $existing = @($settings.hooks.$eventName)
    }
    # Replace only this installer's prior entries, keeping all unrelated hooks.
    $kept = @()
    foreach ($entry in $existing) {
        $containsMochi = $false
        if ($entry.hooks) {
            foreach ($h in @($entry.hooks)) {
                if ($h.command -and $h.command -like '*mochi-hook.ps1*') { $containsMochi = $true }
            }
        }
        if (-not $containsMochi) { $kept += $entry }
    }
    $timeout = if ($eventName -eq 'PreToolUse') { 125 } else { 10 }
    $newEntry = [pscustomobject]@{
        matcher = '.*'
        hooks = @([pscustomobject]@{ type = 'command'; command = $command; timeout = $timeout })
    }
    $settings.hooks | Add-Member -NotePropertyName $eventName -NotePropertyValue @($kept + $newEntry) -Force
}
$settings | ConvertTo-Json -Depth 100 | Set-Content -Path $settingsPath -Encoding UTF8
Write-Host "Mochi Island hooks installed into $settingsPath" -ForegroundColor Green
Write-Host 'A timestamped backup was created. Keep Mochi Island running to receive hook events.' -ForegroundColor Cyan
