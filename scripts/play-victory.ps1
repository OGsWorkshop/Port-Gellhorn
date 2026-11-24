# Launches the game on the VIctory-based runtime from your own unpacked Xbox Series dump.
# Usage: .\scripts\play-victory.ps1 -Game "D:\path\to\dump" [-AutoInput] [-DebugView] [-Target 2560x1440-f28]
param(
    [Parameter(Mandatory)] [string]$Game,
    # Press A on a timer so the title screens advance without a controller.
    [switch]$AutoInput,
    # Show one of the game's internal geometry buffers in the window (debug colours).
    [switch]$DebugView,
    # Which internal render target to show with -DebugView, as WxH-fN (width x height - frame N).
    [string]$Target = '2560x1440-f28'
)
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$vic  = Join-Path $root 'runtime\victory\target\release\victory.exe'

if (-not (Test-Path $vic)) { throw 'Not built yet. Run .\scripts\setup-victory.ps1 first.' }
$exe = Join-Path $Game 'game_scarlett_master.exe'
if (-not (Test-Path $exe)) {
    throw "No game_scarlett_master.exe in '$Game'. Point -Game at your unpacked dump."
}

$env:VICTORY_GAME_DIR = $Game
$env:VICTORY_SHADER_LOOP_LIMIT = '32'
# Write button names here ("A", "DOWN DOWN A") to press them while the game runs.
$env:VICTORY_PRESS_FILE = Join-Path $root 'press.txt'
Set-Content -Path $env:VICTORY_PRESS_FILE -Value '' -NoNewline
if ($AutoInput) { $env:VICTORY_AUTO_INPUT = '1' }
if ($DebugView) {
    $env:VICTORY_DEBUG_PS = '1'
    $env:VICTORY_SHOW_TARGET = $Target
}

$logs = Join-Path $root 'logs'
New-Item -ItemType Directory -Force $logs | Out-Null
$log = Join-Path $logs ("victory-{0:yyyyMMdd-HHmmss}.log" -f (Get-Date))
Write-Host "Starting... log: $log" -ForegroundColor Cyan
# Windows PowerShell 5.1 turns a native program's stderr into a terminating error under 'Stop'
# when it is redirected, which would kill the run on the first log line.
$ErrorActionPreference = 'Continue'
& $vic run $exe --execute *> $log
exit $LASTEXITCODE
