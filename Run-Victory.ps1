<#
.SYNOPSIS
    Runs the bundled VIctory runtime against your own copy of GTA VI.

.DESCRIPTION
    VIctory is included as a git subtree under victory/ with its original history.
    It supplies the Windows runtime and D3D12 presentation layer; Port Gellhorn
    supplies the shader translator, asset tools, and documented console formats.
    Game data, cache and saves stay outside this repository.

.PARAMETER Warp
    Render on the software adapter. Use this if entering the world hangs your GPU.
#>
param(
    [switch]$Warp,
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$Args
)

$ErrorActionPreference = 'Stop'
$root = $PSScriptRoot
$game = 'C:\Users\rhyth\Downloads\GTA6\GTA6.Xbox.v1.0.2.1-CYBERLEEK'
$exe = Join-Path $game 'game_scarlett_master.exe'
$victory = Join-Path $root 'victory\target\release\victory.exe'

if (-not (Test-Path -LiteralPath $victory)) {
    throw "VIctory is not built. Run: cargo build --release -p victory  (from victory/)"
}
if (-not (Test-Path -LiteralPath $exe)) {
    throw "Title executable not found at $exe"
}

$env:VICTORY_GAME_DIR = $game
$env:VICTORY_CACHE_DIR = 'D:\victory-data\cache'
$env:VICTORY_DATA_DIR = 'D:\victory-data\data'
if ($Warp) { $env:VICTORY_D3D12_WARP = '1' }

& $victory run $exe --execute @Args
exit $LASTEXITCODE
