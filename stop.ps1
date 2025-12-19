# GameVault Stop Script for Windows
# Run this in PowerShell: .\stop.ps1

Write-Host "Stopping GameVault..." -ForegroundColor Cyan

$hasPodman = Get-Command podman -ErrorAction SilentlyContinue

if ($hasPodman) {
    # Check for running containers first
    $containers = podman ps -a --filter "name=gamevault" --format "{{.Names}}"

    if ($containers) {
        # Stop containers
        foreach ($container in $containers) {
            Write-Host "Stopping $container..." -ForegroundColor Yellow
            podman stop $container 2>$null
        }

        # Ask if user wants to remove containers
        $remove = Read-Host "`nRemove container(s) too? (y/N)"
        if ($remove -eq 'y' -or $remove -eq 'Y') {
            foreach ($container in $containers) {
                Write-Host "Removing $container..." -ForegroundColor Yellow
                podman rm $container
            }
            Write-Host "`nGameVault stopped and removed!" -ForegroundColor Green
        } else {
            Write-Host "`nGameVault stopped (container kept)." -ForegroundColor Green
        }
    } else {
        Write-Host "No GameVault containers found." -ForegroundColor Yellow
    }
} else {
    Write-Host "Podman not found. Please install Podman Desktop." -ForegroundColor Red
    Write-Host "Download: https://podman-desktop.io/" -ForegroundColor Yellow
}
