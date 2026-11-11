# Kill whatever still listens on a TCP port: a leftover `cargo run` keeps a lock on
# target/debug/scout-beam.exe, and the next build fails with "Access is denied".

param(
    [int]$Port = 8002
)

$listeners = Get-NetTCPConnection -LocalPort $Port -State Listen -ErrorAction SilentlyContinue

if (-not $listeners) {
    Write-Host "nothing listening on $Port"
    exit 0
}

foreach ($pid_ in ($listeners.OwningProcess | Select-Object -Unique)) {
    $process = Get-Process -Id $pid_ -ErrorAction SilentlyContinue
    if (-not $process) {
        Write-Host "port $Port held by PID $pid_, already gone"
        continue
    }

    Write-Host "killing $($process.ProcessName) (PID $($process.Id)) on port $Port"
    Stop-Process -Id $process.Id -Force
}
