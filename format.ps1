# Format all examples/*.branchy with branchy formatter.
# Usage: .\format.ps1
# Runs: docker-compose run --rm --entrypoint cargo app run --bin fmt_all

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $root

Write-Host "Formatting all examples/*.branchy ..." -ForegroundColor Cyan
$out = docker-compose run --rm --entrypoint "cargo" app run --bin fmt_all 2>&1
$code = $LASTEXITCODE
$out | ForEach-Object { Write-Host $_ }
if ($code -ne 0) {
    Write-Host "Format failed (exit $code)." -ForegroundColor Red
    exit $code
}
Write-Host "Done." -ForegroundColor Green
