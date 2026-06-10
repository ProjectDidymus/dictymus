param(
    [Parameter(Mandatory)] [string]$Binary,
    [Parameter(Mandatory)] [string]$Fixture
)
$ErrorActionPreference = "Stop"

# Launch with fixture path (quoted to handle spaces).
$proc = Start-Process -FilePath $Binary -ArgumentList "`"$Fixture`"" -PassThru
$appPid = $proc.Id
Write-Host "Launched PID $appPid"

try {
    # Wait for window to appear via list-windows (OS-level, no UIA tree traversal).
    $hwnd = $null
    $deadline = (Get-Date).AddSeconds(30)
    while ($null -eq $hwnd -and (Get-Date) -lt $deadline) {
        $wins = winapp ui list-windows -a $appPid --json 2>$null | ConvertFrom-Json
        if ($wins.Count -gt 0) { $hwnd = $wins[0].hwnd }
        if ($null -eq $hwnd) { Start-Sleep -Milliseconds 500 }
    }
    if ($null -eq $hwnd) { throw "Window did not appear within 30s" }
    Write-Host "Window HWND: $hwnd"

    # Get window slug using -a PID (not -w HWND: -w hangs when window lacks focus).
    $winSlug = $null
    $deadline2 = (Get-Date).AddSeconds(10)
    while ($null -eq $winSlug -and (Get-Date) -lt $deadline2) {
        $line = (winapp ui inspect -a $appPid --depth 1 2>&1 | Select-String "^win-" | Select-Object -First 1)
        if ($line) { $winSlug = $line.Line.Trim().Split(" ")[0] }
        if ($null -eq $winSlug) { Start-Sleep -Milliseconds 500 }
    }
    if ($null -eq $winSlug) { throw "Could not get window slug within 10s" }
    Write-Host "Window slug: $winSlug"

    # Confirm window is accessible.
    winapp ui wait-for $winSlug -a $appPid -t 5000

    # Wait for dictionary tab to load (use -a PID, not -w HWND).
    $tabFound = $false
    $deadline3 = (Get-Date).AddSeconds(20)
    while (-not $tabFound -and (Get-Date) -lt $deadline3) {
        $tabLine = winapp ui inspect -a $appPid --depth 3 2>&1 | Select-String "TabItem"
        if ($tabLine) { $tabFound = $true } else { Start-Sleep -Milliseconds 500 }
    }
    if (-not $tabFound) { throw "Dictionary tab did not appear within 20s" }
    Write-Host "Tab loaded."

    Start-Sleep -Milliseconds 500

    # Type into search field (control ID -31988 = Edit "Search", stable across sessions).
    winapp ui set-value "-31988" "a" -a $appPid
    Write-Host "Search set."
    Start-Sleep -Milliseconds 500

    # Click list to select first item (control ID -31984 = List "Lemmas").
    winapp ui click "-31984" -a $appPid
    Write-Host "List clicked."
    Start-Sleep -Milliseconds 1000

    winapp ui screenshot -a $appPid -o winapp-tests/smoke.png
    Write-Host "Smoke test passed."
}
finally {
    Stop-Process -Id $appPid -ErrorAction SilentlyContinue
}
