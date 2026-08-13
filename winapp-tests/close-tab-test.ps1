param(
    [Parameter(Mandatory)] [string]$Binary,
    [Parameter(Mandatory)] [string]$Fixture
)
$ErrorActionPreference = "Stop"

# Keep the update dialog from stealing focus mid-test.
$env:DICTYMUS_NO_UPDATE_CHECK = "1"

$stderrFile = Join-Path $env:TEMP "dictymus-close-test-stderr.txt"
Remove-Item $stderrFile -ErrorAction SilentlyContinue

$proc = Start-Process -FilePath $Binary -ArgumentList "`"$Fixture`"" -PassThru -RedirectStandardError $stderrFile
$appPid = $proc.Id
Write-Host "Launched PID $appPid"

try {
    # Wait for window.
    $hwnd = $null
    $deadline = (Get-Date).AddSeconds(30)
    while ($null -eq $hwnd -and (Get-Date) -lt $deadline) {
        $wins = winapp ui list-windows -a $appPid --json 2>$null | ConvertFrom-Json
        if ($wins.Count -gt 0) { $hwnd = $wins[0].hwnd }
        if ($null -eq $hwnd) { Start-Sleep -Milliseconds 500 }
    }
    if ($null -eq $hwnd) { throw "Window did not appear within 30s" }
    Write-Host "Window HWND: $hwnd"

    # Wait for the dictionary tab.
    $tabFound = $false
    $deadline3 = (Get-Date).AddSeconds(20)
    while (-not $tabFound -and (Get-Date) -lt $deadline3) {
        $tabLine = winapp ui inspect -a $appPid --depth 3 2>&1 | Select-String "TabItem"
        if ($tabLine) { $tabFound = $true } else { Start-Sleep -Milliseconds 500 }
    }
    if (-not $tabFound) { throw "Dictionary tab did not appear within 20s" }
    Write-Host "Tab loaded."

    # Send Ctrl+F4 to close the tab.
    $wsh = New-Object -ComObject WScript.Shell
    if (-not $wsh.AppActivate($appPid)) { throw "Could not activate window" }
    Start-Sleep -Milliseconds 500
    $wsh.SendKeys("^{F4}")
    Write-Host "Sent Ctrl+F4."
    Start-Sleep -Seconds 2

    # Tab must be gone.
    $tabLine = winapp ui inspect -a $appPid --depth 3 2>&1 | Select-String "TabItem"
    if ($tabLine) { throw "Tab still present after Ctrl+F4" }
    Write-Host "Tab closed."

    # App must still be responsive: inspect succeeds and process is alive.
    $null = winapp ui inspect -a $appPid --depth 1 2>&1
    if ($proc.HasExited) { throw "App exited unexpectedly" }
    Write-Host "App responsive after close."

    # Reopen via menu would need a file dialog; instead verify clean exit.
    $wsh.SendKeys("%{F4}")  # Alt+F4 closes the frame
    if (-not $proc.WaitForExit(10000)) { throw "App did not exit on Alt+F4 (frozen?)" }
    Write-Host "App exited cleanly (code $($proc.ExitCode))."
}
finally {
    Stop-Process -Id $appPid -ErrorAction SilentlyContinue
}

# Drop impl proof: stderr must show the tab was deallocated.
$stderr = Get-Content $stderrFile -Raw -ErrorAction SilentlyContinue
if ($stderr -match "DictionaryTab dropped") {
    Write-Host "DictionaryTab Drop confirmed: $($stderr.Trim())"
    Write-Host "Close-tab test passed."
} else {
    throw "No 'DictionaryTab dropped' in stderr — tab leaked. stderr: $stderr"
}
