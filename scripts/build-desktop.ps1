# Build Lightmatter desktop (Tauri) with MSVC environment.
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

$vcvars = "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
if (-not (Test-Path $vcvars)) {
    throw "MSVC Build Tools not found at $vcvars"
}

$sac = (Get-ItemProperty "HKLM:\SYSTEM\CurrentControlSet\Control\CI\Policy" -ErrorAction SilentlyContinue).VerifiedAndReputablePolicyState
if ($sac -eq 1) {
    Write-Warning "Smart App Control is ON (state=$sac). Local Rust builds will fail until it is Off and you reboot."
}

cmd /c "`"$vcvars`" && cargo tauri build"
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "Bundle output:"
Get-ChildItem -Recurse "$root\src-tauri\target\release\bundle" -ErrorAction SilentlyContinue |
    Select-Object FullName, Length
