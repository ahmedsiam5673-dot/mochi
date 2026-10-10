# Mochi Island local hook forwarder. No credentials are sent.
# Claude Code calls this script with JSON on stdin. It forwards to a loopback-only listener.
$ErrorActionPreference = 'Stop'
$inputJson = [Console]::In.ReadToEnd()
if ([string]::IsNullOrWhiteSpace($inputJson)) { $inputJson = '{}' }
try {
    $response = Invoke-WebRequest -UseBasicParsing -Uri 'http://127.0.0.1:49321/hook' -Method Post -ContentType 'application/json' -Body $inputJson -TimeoutSec 125
    if ($response.Content) { [Console]::Out.WriteLine($response.Content) }
    else { [Console]::Out.WriteLine('{}') }
} catch {
    # Fail open if Mochi is not running: do not permanently block the user's agent.
    [Console]::Out.WriteLine('{}')
}
