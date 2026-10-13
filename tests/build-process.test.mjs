import assert from 'node:assert/strict'
import { test } from 'node:test'
import { spawnSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'

test('build process check only stops the matching executable and reports inspection/stop failures', {
  skip: process.platform !== 'win32',
}, () => {
  const script = fileURLToPath(new URL('../scripts/stop-build-exe.ps1', import.meta.url))
  const quote = value => "'" + value.replaceAll("'", "''") + "'"
  const target = "C:\\build & (test)!\\it's a test\\pingport.exe"
  const cases = [
    { name: 'no process', processes: '@()', status: 0 },
    { name: 'different executable', path: "'C:\\other\\pingport.exe'", status: 0 },
    { name: 'matching executable', path: '$env:PINGPORT_BUILD_EXE.ToUpperInvariant()', stop: true, status: 0 },
    { name: 'exited process', path: '$null', exited: true, status: 0 },
    { name: 'unavailable path', path: '$null', status: 1, error: 'Cannot inspect PingPort PID' },
    { name: 'path access denied', pathThrows: true, status: 1, error: 'Cannot inspect PingPort PID' },
    { name: 'stop denied', path: '$env:PINGPORT_BUILD_EXE', stop: true, stopThrows: true, status: 1, error: 'Access denied' },
    { name: 'stop timeout', path: '$env:PINGPORT_BUILD_EXE', stop: true, timeout: true, status: 1, error: 'Timed out' },
  ]
  for (const scenario of cases) {
    // Shadow process commands in the child PowerShell: no real process is stopped.
    const command = `
      function Get-Process {
        $fake = [pscustomobject]@{ ProcessName = 'pingport'; Id = 123; HasExited = $${!!scenario.exited}; Path = ${scenario.path ?? '$null'} }
        ${scenario.pathThrows ? "$fake.PSObject.Properties.Remove('Path'); $fake | Add-Member ScriptProperty Path { throw 'Access denied' }" : ''}
        $fake | Add-Member ScriptMethod WaitForExit { param($milliseconds) return $${!scenario.timeout} }
        return ${scenario.processes ?? '$fake'}
      }
      function Stop-Process { param($InputObject, [switch]$Force, $ErrorAction)
        Write-Output 'MOCK_STOP'
        ${scenario.stopThrows ? "throw 'Access denied'" : ''}
      }
      & ${quote(script)}
    `
    const result = spawnSync('powershell.exe', [
      '-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass',
      '-EncodedCommand', Buffer.from(command, 'utf16le').toString('base64'),
    ], { encoding: 'utf8', env: { ...process.env, PINGPORT_BUILD_EXE: target }, timeout: 20000, windowsHide: true })
    assert.ifError(result.error)
    assert.equal(result.status, scenario.status, `${scenario.name}: ${result.stdout}\n${result.stderr}`)
    assert.equal(result.stdout.includes('MOCK_STOP'), !!scenario.stop, scenario.name)
    if (scenario.error) assert.ok(result.stdout.includes(scenario.error), `${scenario.name}: ${result.stdout}`)
  }
})
