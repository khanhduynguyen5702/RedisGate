@echo off
title RedisGate - Quick Start
color 0A
cls

echo.
echo ================================================
echo   RedisGate - Redis API Key Setup Complete!
echo ================================================
echo.

cd /d K:\RedisGate

echo [1/3] Setting environment variables...
set DATABASE_URL=postgresql://redisgate_dev:redisgate_dev@localhost:5432/redisgate_dev
set JWT_SECRET=redisgate-super-secret-key-2024-change-this-in-production
set APP_PORT=3000
set RUST_LOG=info

echo [2/3] Starting server...
echo.
echo Server will start on: http://localhost:3000
echo.

start /b target\release\redisgate.exe 2>&1

echo [3/3] Waiting for server to start...
timeout /t 5 /nobreak >nul

echo.
echo ================================================
echo   NEXT STEPS:
echo ================================================
echo.
echo 1. Open browser: http://localhost:3000
echo 2. Login with: test@example.com / Password123!
echo 3. Go to Settings tab
echo 4. Copy your Redis API Key
echo 5. Use it to connect to instances!
echo.
echo Full guide: REDIS_API_KEY_GUIDE.md
echo.
echo ================================================
echo.

pause

