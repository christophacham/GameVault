#!/usr/bin/env pwsh
# =============================================================================
# GameVault Portable Windows Executable Builder
# =============================================================================
# Uses Podman to cross-compile a portable Windows executable
# Usage: ./build-portable.ps1

$ErrorActionPreference = "Stop"

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "  GameVault Portable Build" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

# Check if Podman is available
if (-not (Get-Command podman -ErrorAction SilentlyContinue)) {
    Write-Error "Podman is not installed or not in PATH"
    exit 1
}

# Build the Windows executable using Dockerfile.windows
Write-Host "[1/3] Building Windows executable with Podman..." -ForegroundColor Yellow
Write-Host "      This may take several minutes on first run.`n" -ForegroundColor Gray

podman build -f Dockerfile.windows -t gamevault-windows-builder .

if ($LASTEXITCODE -ne 0) {
    Write-Error "Build failed!"
    exit 1
}

# Create distribution directory
Write-Host "`n[2/3] Extracting build artifacts..." -ForegroundColor Yellow

$distDir = "dist"
if (Test-Path $distDir) {
    Remove-Item -Recurse -Force $distDir
}
New-Item -ItemType Directory -Path $distDir | Out-Null
New-Item -ItemType Directory -Path "$distDir/data" | Out-Null
New-Item -ItemType Directory -Path "$distDir/cache" | Out-Null
New-Item -ItemType Directory -Path "$distDir/logs" | Out-Null

# Extract the executable from the container
$containerId = podman create gamevault-windows-builder
podman cp "${containerId}:/output/GameVault.exe" "$distDir/GameVault.exe"
podman rm $containerId | Out-Null

# Copy config template
Copy-Item "config.example.toml" "$distDir/config.toml"

# Report results
Write-Host "`n[3/3] Build complete!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan

if (Test-Path "$distDir/GameVault.exe") {
    $exeSize = [math]::Round((Get-Item "$distDir/GameVault.exe").Length / 1MB, 2)
    Write-Host "`nOutput directory: $distDir/" -ForegroundColor White
    Write-Host "Executable size:  $exeSize MB" -ForegroundColor White
    Write-Host "`nContents:" -ForegroundColor Gray
    Get-ChildItem $distDir | ForEach-Object { Write-Host "  - $($_.Name)" -ForegroundColor Gray }

    Write-Host "`nTo run GameVault:" -ForegroundColor Yellow
    Write-Host "  1. Edit $distDir/config.toml to set your games path" -ForegroundColor White
    Write-Host "  2. Run $distDir/GameVault.exe" -ForegroundColor White
    Write-Host "  3. Browser will open automatically to http://localhost:3000" -ForegroundColor White
} else {
    Write-Error "GameVault.exe was not created!"
    exit 1
}

Write-Host "`n========================================`n" -ForegroundColor Cyan
