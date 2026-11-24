# Builds the runtime and fetches the D3D12 Agility SDK (needed on Windows 10).
# Usage: .\scripts\setup.ps1
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$bin  = Join-Path $root 'runtime\xcl\target\release'

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw 'Rust is not installed. Get it from https://rustup.rs (stable-x86_64-pc-windows-msvc).'
}

Write-Host '==> Building XCL runtime (release)...' -ForegroundColor Cyan
Push-Location (Join-Path $root 'runtime\xcl')
cargo build --release
Pop-Location

$agility = Join-Path $bin 'D3D12'
if (-not (Test-Path (Join-Path $agility 'D3D12Core.dll'))) {
    Write-Host '==> Downloading D3D12 Agility SDK 1.614.1...' -ForegroundColor Cyan
    $tmp = Join-Path $env:TEMP 'agility-sdk'
    Remove-Item $tmp -Recurse -Force -ErrorAction SilentlyContinue
    New-Item -ItemType Directory -Force $tmp | Out-Null
    $pkg = Join-Path $tmp 'agility.zip'
    Invoke-WebRequest 'https://www.nuget.org/api/v2/package/Microsoft.Direct3D.D3D12/1.614.1' -OutFile $pkg
    Expand-Archive $pkg -DestinationPath $tmp -Force
    New-Item -ItemType Directory -Force $agility | Out-Null
    Copy-Item (Join-Path $tmp 'build\native\bin\x64\D3D12Core.dll') $agility
    Copy-Item (Join-Path $tmp 'build\native\bin\x64\d3d12SDKLayers.dll') $agility
}

Write-Host ''
Write-Host 'Done. Run the game with:' -ForegroundColor Green
Write-Host '  .\scripts\play.ps1 -Game "D:\path\to\your\GTA VI dump"'
