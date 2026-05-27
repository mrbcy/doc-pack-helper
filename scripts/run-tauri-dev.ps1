$ErrorActionPreference = "Stop"

$repoRoot = "D:\source\Projects\doc-pack-helper"
$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
$mingwBin = Join-Path $env:USERPROFILE ".local\toolchains\mingw64\bin"

$env:Path = "$cargoBin;$mingwBin;$env:Path"
$env:RUSTUP_TOOLCHAIN = "stable-x86_64-pc-windows-gnu"

Set-Location $repoRoot

Write-Host "Using node: $(node -v)"
Write-Host "Using npm: $(npm -v)"
Write-Host "Using cargo: $(cargo -V)"
Write-Host "Using gcc: $(& x86_64-w64-mingw32-gcc --version | Select-Object -First 1)"

npm exec tauri dev
