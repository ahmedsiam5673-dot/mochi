[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$settingsPath = Join-Path $HOME '.claude\settings.json'
if (-not (Test-Path $settingsPath)) { Write-Host 'Claude Code settings.json was not found.'; exit 0 }
$settings = (Get-Content $settingsPath -Raw) | ConvertFrom-Json
if (-not $settings.hooks) { Write-Host 'No hooks are configured.'; exit 0 }
foreach ($eventName in @($settings.hooks.PSObject.Properties.Name)) {
    $kept = @()
    foreach ($entry in @($settings.hooks.$eventName)) {
        $containsMochi = $false
        if ($entry.hooks) {
            foreach ($h in @($entry.hooks)) {
                if ($h.command -and $h.command -like '*mochi-hook.ps1*') { $containsMochi = $true }
            }
        }
        if (-not $containsMochi) { $kept += $entry }
    }
    if ($kept.Count -eq 0) { $settings.hooks.PSObject.Properties.Remove($eventName) }
    else { $settings.hooks | Add-Member -NotePropertyName $eventName -NotePropertyValue $kept -Force }
}
$settings | ConvertTo-Json -Depth 100 | Set-Content -Path $settingsPath -Encoding UTF8
Write-Host 'Mochi hook entries removed. All unrelated hooks were preserved.' -ForegroundColor Green
