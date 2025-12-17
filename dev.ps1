# GameVault Development Script
# Runs backend and frontend in separate terminals

Write-Host "GameVault Development Mode" -ForegroundColor Cyan
Write-Host "=========================" -ForegroundColor Cyan

# Create data directory if it doesn't exist
if (-not (Test-Path "data")) {
    New-Item -ItemType Directory -Path "data" | Out-Null
}

# Set environment variables
$env:DATABASE_URL = "sqlite:./data/games.db?mode=rwc"
$env:GAMES_PATH = "F:/Games"
$env:RUST_LOG = "debug"
$env:PORT = "8080"

Write-Host "`nStarting backend on port 8080..." -ForegroundColor Yellow
Start-Process powershell -ArgumentList "-NoExit", "-Command", "cd backend; cargo run"

Write-Host "Starting frontend on port 3000..." -ForegroundColor Yellow
Start-Process powershell -ArgumentList "-NoExit", "-Command", "cd frontend; npm run dev"

Write-Host "`nDevelopment servers starting!" -ForegroundColor Green
Write-Host "Backend API: http://localhost:8080/api" -ForegroundColor Cyan
Write-Host "Frontend:    http://localhost:3000" -ForegroundColor Cyan
