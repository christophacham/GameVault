# GameVault Start Script for Windows
# Run this in PowerShell: .\start.ps1

Write-Host "Starting GameVault..." -ForegroundColor Cyan

$hasPodman = Get-Command podman -ErrorAction SilentlyContinue

if ($hasPodman) {
    # Check if Podman machine is running
    $machineStatus = podman machine list --format "{{.Running}}" 2>$null | Select-Object -First 1
    if ($machineStatus -ne "true") {
        Write-Host "Starting Podman machine..." -ForegroundColor Yellow
        podman machine start
    }

    # Check if image exists
    $imageExists = podman images gamevault:latest -q

    if (-not $imageExists) {
        Write-Host "Building Podman image first..." -ForegroundColor Yellow
        podman build -t gamevault:latest .
    }

    Write-Host "Starting with Podman Compose..." -ForegroundColor Yellow
    podman compose up -d

    if ($LASTEXITCODE -eq 0) {
        Write-Host "`nGameVault is running!" -ForegroundColor Green
        Write-Host "Access at: http://localhost:3000" -ForegroundColor Cyan
        Write-Host "`nCommands:" -ForegroundColor Yellow
        Write-Host "  podman compose logs -f    # View logs" -ForegroundColor White
        Write-Host "  podman compose down       # Stop" -ForegroundColor White
    }
} else {
    Write-Host "Podman not found. Please install Podman Desktop." -ForegroundColor Red
    Write-Host "Download: https://podman-desktop.io/" -ForegroundColor Yellow
}
