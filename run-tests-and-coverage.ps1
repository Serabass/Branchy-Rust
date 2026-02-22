# Run tests then coverage. Tests in Docker; coverage locally (tarpaulin needs Rust 1.85+).
# Usage: .\run-tests-and-coverage.ps1

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $root

Write-Host "=== Running tests ===" -ForegroundColor Cyan
docker-compose run --rm test
if ($LASTEXITCODE -ne 0) {
    Write-Host "Tests failed. Aborting." -ForegroundColor Red
    exit $LASTEXITCODE
}

Write-Host "`n=== Coverage (Docker, first run may install tarpaulin) ===" -ForegroundColor Cyan
if (-not (Test-Path "coverage")) {
    New-Item -ItemType Directory -Path "coverage" | Out-Null
}
docker-compose run --rm coverage
if ($LASTEXITCODE -ne 0) {
    Write-Host "Coverage failed." -ForegroundColor Red
    exit $LASTEXITCODE
}
Write-Host "`nDone. Open coverage/html/index.html for report." -ForegroundColor Green
