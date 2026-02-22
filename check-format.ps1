# Check that all examples/*.branchy are formatted (CI-style).
# Exit 1 if any file would be reformatted.
# Usage: .\check-format.ps1
# Runs: docker-compose run --rm --entrypoint cargo app run --bin check_fmt

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $root

Write-Host "Checking formatting of examples/*.branchy ..." -ForegroundColor Cyan
$out = docker-compose run --rm --entrypoint "cargo" app run --bin check_fmt 2>&1
$code = $LASTEXITCODE
$out | ForEach-Object { Write-Host $_ }
if ($code -ne 0) {
    Write-Host "Check failed: some files need formatting (run .\format.ps1)." -ForegroundColor Red
    exit $code
}
Write-Host "All files are formatted." -ForegroundColor Green
