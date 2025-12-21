#!/usr/bin/env pwsh
# =============================================================================
# GameVault Portable Windows Executable Builder
# =============================================================================
# Usage: ./build-portable.ps1
# Output: dist/GameVault.exe (ready to run)

$ErrorActionPreference = "Stop"

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "  GameVault Portable Build" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

# Build with Podman
Write-Host "[1/2] Building Windows executable..." -ForegroundColor Yellow
podman build -f Dockerfile.windows -t gamevault-windows-builder . --quiet

if ($LASTEXITCODE -ne 0) { Write-Error "Build failed!"; exit 1 }

# Extract to dist/
Write-Host "[2/2] Extracting to dist/..." -ForegroundColor Yellow

if (Test-Path dist) { Remove-Item -Recurse -Force dist }
New-Item -ItemType Directory -Path dist, dist/data, dist/cache, dist/logs | Out-Null

$id = podman create gamevault-windows-builder
podman cp "${id}:/output/GameVault.exe" dist/
podman rm $id | Out-Null

Copy-Item config.example.toml dist/config.toml

# Done
$size = [math]::Round((Get-Item dist/GameVault.exe).Length / 1MB, 1)
Write-Host "`n========================================" -ForegroundColor Green
Write-Host "  Build complete! ($size MB)" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host "`nTo run: .\dist\GameVault.exe" -ForegroundColor White
Write-Host "Config: dist\config.toml`n" -ForegroundColor Gray
