# GameVault Build Script for Windows
# Run this in PowerShell: .\build.ps1

Write-Host "GameVault Build Script" -ForegroundColor Cyan
Write-Host "======================" -ForegroundColor Cyan

# Check prerequisites
Write-Host "`nChecking prerequisites..." -ForegroundColor Yellow

$hasPodman = Get-Command podman -ErrorAction SilentlyContinue
$hasNode = Get-Command node -ErrorAction SilentlyContinue
$hasCargo = Get-Command cargo -ErrorAction SilentlyContinue

if ($hasPodman) {
    Write-Host "[OK] Podman found" -ForegroundColor Green
    # Check if Podman machine is running
    $machineStatus = podman machine list --format "{{.Running}}" 2>$null | Select-Object -First 1
    if ($machineStatus -ne "true") {
        Write-Host "[!] Podman machine not running - starting it..." -ForegroundColor Yellow
        podman machine start
    }
} else {
    Write-Host "[!] Podman not found - needed for containerized build" -ForegroundColor Yellow
}

if ($hasNode) {
    Write-Host "[OK] Node.js found" -ForegroundColor Green
} else {
    Write-Host "[!] Node.js not found - needed for frontend dev" -ForegroundColor Yellow
}

if ($hasCargo) {
    Write-Host "[OK] Rust/Cargo found" -ForegroundColor Green
} else {
    Write-Host "[!] Rust/Cargo not found - needed for backend dev" -ForegroundColor Yellow
}

# Build options
Write-Host "`nBuild Options:" -ForegroundColor Cyan
Write-Host "1. Podman build (recommended)" -ForegroundColor White
Write-Host "2. Local development build" -ForegroundColor White
Write-Host "3. Exit" -ForegroundColor White

$choice = Read-Host "`nSelect option"

switch ($choice) {
    "1" {
        if (-not $hasPodman) {
            Write-Host "Podman is required for this option!" -ForegroundColor Red
            exit 1
        }
        Write-Host "`nBuilding Podman image..." -ForegroundColor Yellow
        podman build -t gamevault:latest .
        if ($LASTEXITCODE -eq 0) {
            Write-Host "`nBuild successful! Run with:" -ForegroundColor Green
            Write-Host "podman compose up -d" -ForegroundColor Cyan
            Write-Host "`nAccess at: http://localhost:3000" -ForegroundColor Green
        }
    }
    "2" {
        Write-Host "`nBuilding locally..." -ForegroundColor Yellow

        # Build backend
        if ($hasCargo) {
            Write-Host "Building Rust backend..." -ForegroundColor Yellow
            Push-Location backend
            cargo build --release
            Pop-Location
        }

        # Build frontend
        if ($hasNode) {
            Write-Host "Building Next.js frontend..." -ForegroundColor Yellow
            Push-Location frontend
            npm install
            npm run build
            Pop-Location

            # Copy frontend to backend public folder
            if (Test-Path "backend/target/release") {
                Copy-Item -Recurse -Force frontend/out/* backend/public/
            }
        }

        Write-Host "`nBuild complete!" -ForegroundColor Green
    }
    "3" {
        Write-Host "Exiting..." -ForegroundColor Yellow
        exit 0
    }
    default {
        Write-Host "Invalid option" -ForegroundColor Red
    }
}
