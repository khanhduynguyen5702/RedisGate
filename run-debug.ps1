$env:DATABASE_URL = "postgresql://redisgate_dev:redisgate_dev@localhost:5432/redisgate_dev"
$env:JWT_SECRET = "redisgate-super-secret-key-2024-change-this-in-production"
$env:APP_PORT = "3000"

Write-Host "============================================" -ForegroundColor Cyan
Write-Host "  RedisGate Server - Debug Mode" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan
Write-Host ""

Write-Host "[CONFIG] Environment:" -ForegroundColor Yellow
Write-Host "  DATABASE_URL: postgresql://redisgate_dev:***@localhost:5432/redisgate_dev"
Write-Host "  JWT_SECRET: ***configured***"
Write-Host "  APP_PORT: 3000"
Write-Host ""

Write-Host "[START] Starting server..." -ForegroundColor Yellow
Write-Host ""

Set-Location "K:\RedisGate"

& ".\target\release\redisgate.exe"

