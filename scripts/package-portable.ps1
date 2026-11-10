$ErrorActionPreference = "Stop"

$root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$backend = Join-Path $root "target/x86_64-pc-windows-msvc/release/lightmatter.exe"
if (-not (Test-Path $backend)) {
    throw "No se encontró el backend Windows compilado: $backend"
}

$dist = Join-Path $root "dist"
$portable = Join-Path $dist "lightmatter-portable"
$public = Join-Path $portable "public"
$source = Join-Path $portable "src"
$archive = Join-Path $dist "lightmatter-portable.zip"

Remove-Item -LiteralPath $portable -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item -LiteralPath $archive -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $public, $source, (Join-Path $portable "data") | Out-Null

Copy-Item -Path (Join-Path $root "public/*") -Destination $public -Recurse -Force
Copy-Item -LiteralPath (Join-Path $root "src/pages") -Destination $source -Recurse -Force
Copy-Item -LiteralPath $backend -Destination (Join-Path $portable "lightmatter.exe")

@'
@echo off
cd /d "%~dp0"
set "RESUMA_ADDR=127.0.0.1:4321"
set "RESUMA_DATA_DIR=%~dp0data"
start "" "%~dp0lightmatter.exe"
timeout /t 2 /nobreak >nul
start "" "http://127.0.0.1:4321"
'@ | Set-Content -LiteralPath (Join-Path $portable "start-lightmatter.bat") -Encoding Ascii

Compress-Archive -Path $portable -DestinationPath $archive -Force
Write-Output "Paquete creado: $archive"