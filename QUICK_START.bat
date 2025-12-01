@echo off
echo.
echo ============================================
echo   RedisGate - Quick Start
echo ============================================
echo.

echo [1/3] Checking Redis...
powershell -Command "$client = New-Object System.Net.Sockets.TcpClient; try { $client.Connect('127.0.0.1', 6379); $client.Close(); Write-Host '   OK: Redis is running' -ForegroundColor Green } catch { Write-Host '   STARTING Redis...' -ForegroundColor Yellow; docker run -d --name redis-local -p 6379:6379 redis:7-alpine >$null 2>&1; Start-Sleep -Seconds 3; Write-Host '   OK: Redis started' -ForegroundColor Green }"

echo.
echo [2/3] Building RedisGate...
cargo build --bin redisgate --quiet

echo.
echo [3/3] Starting RedisGate Server...
echo.
echo ============================================
echo   Server will start at:
echo   http://localhost:3000
echo.
echo   Press Ctrl+C to stop
echo ============================================
echo.

cargo run --bin redisgate

pause

