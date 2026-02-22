# Build images with docker buildx bake and optional push to reg.serabass.kz/vibecoding
# Usage:
#   .\build.ps1           - build all (gateway, backend)
#   .\build.ps1 -Push     - build and push to registry
#   .\build.ps1 -Target gateway  - build only gateway (frontend)
#   .\build.ps1 -Target backend   - build only backend
#   .\build.ps1 -Tag v1.0        - tag (default: latest)

param(
    [switch]$Push,
    [string]$Target = "default",
    [string]$Tag = "latest"
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $root

$env:TAG = $Tag
$bakeArgs = @("buildx", "bake", "-f", "docker-bake.hcl")
if ($Push) {
    $bakeArgs += "--push"
}
$bakeArgs += $Target

Write-Host "Building: reg.serabass.kz/vibecoding, tag $Tag, target $Target" -ForegroundColor Cyan
& docker $bakeArgs
if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed." -ForegroundColor Red
    exit $LASTEXITCODE
}
Write-Host "Done." -ForegroundColor Green
