$ErrorActionPreference = 'Stop'

try {
    if ([string]::IsNullOrWhiteSpace($env:PINGPORT_BUILD_EXE)) {
        throw 'PINGPORT_BUILD_EXE is not set.'
    }
    $target = [System.IO.Path]::GetFullPath($env:PINGPORT_BUILD_EXE)
    $processName = [System.IO.Path]::GetFileNameWithoutExtension($target)
    # Enumerating all processes avoids suppressing access errors together with
    # the normal "no process with this name" error from Get-Process -Name.
    $running = @(Get-Process | Where-Object { $_.ProcessName -ieq $processName })
    foreach ($process in $running) {
        if ($process.HasExited) { continue }
        try {
            $processPath = $process.Path
            if ([string]::IsNullOrWhiteSpace($processPath)) {
                throw 'Executable path is unavailable.'
            }
        } catch {
            if ($process.HasExited) { continue }
            throw ('Cannot inspect PingPort PID ' + $process.Id + '. Close it manually and retry. ' + $_.Exception.Message)
        }
        if ($processPath -ine $target) { continue }
        Write-Host ('Stopping PingPort PID ' + $process.Id + '...')
        try {
            Stop-Process -InputObject $process -Force -ErrorAction Stop
            if (-not $process.WaitForExit(10000)) {
                throw ('Timed out waiting for PID ' + $process.Id)
            }
        } catch {
            if ($process.HasExited) { continue }
            throw
        }
    }
    exit 0
} catch {
    Write-Host ('[X] Build process check failed: ' + $_.Exception.Message)
    exit 1
}
