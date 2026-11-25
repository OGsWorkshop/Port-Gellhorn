# Launches a title on the VIctory-based runtime from your own dump (you need your own legally
# obtained copy; nothing is included in this repository).
# Usage: .\scripts\play-victory.ps1 -Game "D:\path\to\dump" [-AutoInput] [-Console series-s|series-x]
#          [-NoOwnUpscaler] [-KeepWatchdog] [-FastBoot] [-SaveSlot name]
#
# Environment it sets (you can also set these by hand before running the exe):
#   VICTORY_GAME_DIR       the dump folder
#   VICTORY_OWN_UPSCALER=1 ask for the title's own upscaler on the Series S profile (recommended)
#   VICTORY_NO_WATCHDOG=1  disarm the title's stall watchdog so it can wait for slow first-run
#                          shader builds instead of ending itself
#   VICTORY_FAST_BOOT=1    with -FastBoot: press through logos and menus (testing only)
#   VICTORY_SAVE_SLOT      with -SaveSlot: pick a save slot by name
#   VICTORY_AUTO_INPUT=1   with -AutoInput: press A on a timer so menus advance without a pad
#   VICTORY_PRESS_FILE     press.txt in the repo root: write button names ("A", "DOWN DOWN A") to press them
param(
    [Parameter(Mandatory)] [string]$Game,
    # The console profile the title sees. series-s is the recommended default.
    [ValidateSet('series-s', 'series-x')] [string]$Console = 'series-s',
    # Press A on a timer so the title screens advance without a controller.
    [switch]$AutoInput,
    # Do not set VICTORY_OWN_UPSCALER=1.
    [switch]$NoOwnUpscaler,
    # Do not set VICTORY_NO_WATCHDOG=1.
    [switch]$KeepWatchdog,
    # Testing only: VICTORY_FAST_BOOT=1.
    [switch]$FastBoot,
    # Save slot name (VICTORY_SAVE_SLOT).
    [string]$SaveSlot = ''
)
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$vic  = Join-Path $root 'runtime\victory\target\release\victory.exe'

if (-not (Test-Path $vic)) { throw 'Not built yet. Run .\scripts\setup-victory.ps1 first.' }
$exe = Join-Path $Game 'game_scarlett_master.exe'
if (-not (Test-Path $exe)) {
    throw "No game_scarlett_master.exe in '$Game'. Point -Game at your own dump."
}

$env:VICTORY_GAME_DIR = $Game
if (-not $NoOwnUpscaler) { $env:VICTORY_OWN_UPSCALER = '1' }
if (-not $KeepWatchdog) { $env:VICTORY_NO_WATCHDOG = '1' }
if ($FastBoot) { $env:VICTORY_FAST_BOOT = '1' }
if ($SaveSlot) { $env:VICTORY_SAVE_SLOT = $SaveSlot }
# Write button names here ("A", "DOWN DOWN A") to press them while the game runs.
$env:VICTORY_PRESS_FILE = Join-Path $root 'press.txt'
Set-Content -Path $env:VICTORY_PRESS_FILE -Value '' -NoNewline
if ($AutoInput) { $env:VICTORY_AUTO_INPUT = '1' }

$logs = Join-Path $root 'logs'
New-Item -ItemType Directory -Force $logs | Out-Null
$log = Join-Path $logs ("victory-{0:yyyyMMdd-HHmmss}.log" -f (Get-Date))
Write-Host "Starting on the $Console profile... log: $log" -ForegroundColor Cyan
# Windows PowerShell 5.1 turns a native program's stderr into a terminating error under 'Stop'
# when it is redirected, which would kill the run on the first log line.
$ErrorActionPreference = 'Continue'
& $vic run $exe --execute --console $Console *> $log
exit $LASTEXITCODE
