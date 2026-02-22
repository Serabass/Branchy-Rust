# Build images, push to registry, restart deployments in Kubernetes (namespace: branchy)
# Usage:
#   .\deploy.ps1           - build all, push, restart all in branchy
#   .\deploy.ps1 -NoBuild  - only rollout restart (skip build/push)
#   .\deploy.ps1 -Tag v1.0 - use custom tag

param(
    [switch]$NoBuild,
    [string]$Tag = "latest",
    [string]$Namespace = "branchy"
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $root

if (-not $NoBuild) {
    Write-Host "Build and push (tag: $Tag)..." -ForegroundColor Cyan
    & "$root\build.ps1" -Push -Tag $Tag
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Build failed." -ForegroundColor Red
        exit $LASTEXITCODE
    }
}

Write-Host "Rollout restart in namespace: $Namespace" -ForegroundColor Cyan
kubectl rollout restart deployment -n $Namespace
if ($LASTEXITCODE -ne 0) {
    Write-Host "kubectl rollout failed." -ForegroundColor Red
    exit $LASTEXITCODE
}

Write-Host "Waiting for rollout..." -ForegroundColor Cyan
kubectl rollout status deployment -n $Namespace --timeout=120s
if ($LASTEXITCODE -ne 0) {
    Write-Host "Rollout status failed or timed out." -ForegroundColor Red
    exit $LASTEXITCODE
}

Write-Host "Done. Deployments in $Namespace restarted." -ForegroundColor Green
