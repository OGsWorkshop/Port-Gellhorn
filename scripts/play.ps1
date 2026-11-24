# Launches GTA VI from your own unpacked Xbox Series dump.
# Usage: .\scripts\play.ps1 -Game "D:\path\to\dump" [-AutoMenu] [-Lazy]
param(
    [Parameter(Mandatory)] [string]$Game,
    # Press A every 3 s from 20 s to 200 s, to walk the menus hands-free.
    [switch]$AutoMenu,
    # Compile shaders on first use instead of up front (faster start, black until compiled).
    [switch]$Lazy
)
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$xcl  = Join-Path $root 'runtime\xcl\target\release\xcl.exe'

if (-not (Test-Path $xcl)) { throw 'Not built yet. Run .\scripts\setup.ps1 first.' }
if (-not (Test-Path (Join-Path $Game 'game_scarlett_master.exe'))) {
    throw "No game_scarlett_master.exe in '$Game'. Point -Game at your unpacked dump."
}

if (-not $Lazy) { $env:XCL_LAZY_PSO = '0' }
if ($AutoMenu)  { $env:XCL_PRESS_A_AT = ((20..200 | Where-Object { ($_ - 20) % 3 -eq 0 }) -join ',') }

$logs = Join-Path $root 'logs'
New-Item -ItemType Directory -Force $logs | Out-Null
$log = Join-Path $logs ("run-{0:yyyyMMdd-HHmmss}.log" -f (Get-Date))
Write-Host "Starting GTA VI... log: $log" -ForegroundColor Cyan
& $xcl run $Game *> $log
