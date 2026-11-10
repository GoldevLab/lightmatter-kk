$ErrorActionPreference = "Stop"

$root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$target = $env:TAURI_ENV_TARGET_TRIPLE
if ([string]::IsNullOrWhiteSpace($target)) {
    $target = ((rustc -vV | Select-String "^host:").ToString() -split "\s+")[1]
}

$debug = $env:TAURI_ENV_DEBUG -eq "true"
$profile = if ($debug) { "debug" } else { "release" }
$args = @(
    "build",
    "--manifest-path", (Join-Path $root "Cargo.toml"),
    "--bin", "lightmatter",
    "--target", $target
)
if (-not $debug) {
    $args += "--release"
}

Push-Location $root
try {
    & cargo @args
    if ($LASTEXITCODE -ne 0) {
        throw "No se pudo compilar el backend Lightmatter para $target."
    }
} finally {
    Pop-Location
}

$extension = if ($target -match "windows") { ".exe" } else { "" }
$backend = Join-Path $root "target/$target/$profile/lightmatter$extension"
if (-not (Test-Path $backend)) {
    throw "No se encontró el backend compilado: $backend"
}

$sidecarName = "lightmatter-$target$extension"
$genericSidecarName = "lightmatter$extension"
$binariesDir = Join-Path $root "src-tauri/binaries"
New-Item -ItemType Directory -Force -Path $binariesDir | Out-Null
Copy-Item -Force $backend (Join-Path $binariesDir $sidecarName)
Copy-Item -Force $backend (Join-Path $binariesDir $genericSidecarName)

$tauriTargetDir = Join-Path $root "src-tauri/target/$target/$profile"
New-Item -ItemType Directory -Force -Path $tauriTargetDir | Out-Null
Copy-Item -Force $backend (Join-Path $tauriTargetDir $sidecarName)
Copy-Item -Force $backend (Join-Path $tauriTargetDir $genericSidecarName)
