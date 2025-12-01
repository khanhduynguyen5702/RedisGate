$env:DATABASE_URL = "postgresql://redisgate_dev:redisgate_dev@localhost:5432/redisgate_dev"
$env:JWT_SECRET = "redisgate-super-secret-key-2024-change-this-in-production"
$env:APP_PORT = "3001"

Write-Host "============================================" -ForegroundColor Green
Write-Host "  RedisGate Server Starting on Port 3001" -ForegroundColor Green
Write-Host "============================================" -ForegroundColor Green
Write-Host ""
Write-Host "Server will be available at:" -ForegroundColor Yellow
Write-Host "  http://localhost:3001" -ForegroundColor Cyan
Write-Host "  http://localhost:3001/token-check.html" -ForegroundColor Cyan
Write-Host ""
Write-Host "Press Ctrl+C to stop the server" -ForegroundColor Gray
Write-Host ""

Set-Location "K:\RedisGate"
& ".\target\release\redisgate.exe"

