@echo off
title RedisGate Server
color 0A

echo.
echo ========================================
echo   RedisGate Server - Starting...
echo ========================================
echo.

cd /d K:\RedisGate

REM Check if server is already running
netstat -ano | findstr ":3000" | findstr "LISTENING" >nul
if %ERRORLEVEL% EQU 0 (
    echo [INFO] Server is already running on port 3000!
    echo.
    echo Opening fix page in browser...
    start http://localhost:3000/fix-401.html
    echo.
    echo ========================================
    echo   Server Status: RUNNING
    echo   Fix Page: http://localhost:3000/fix-401.html
    echo ========================================
    echo.
    pause
    exit /b 0
)

echo [INFO] Starting server...
echo.

REM Set environment variables
set DATABASE_URL=postgresql://redisgate_dev:redisgate_dev@localhost:5432/redisgate_dev
set JWT_SECRET=redisgate-super-secret-key-2024-change-this-in-production
set APP_PORT=3000
set RUST_LOG=info

echo [CONFIG] Environment variables set
echo - DATABASE_URL: postgresql://redisgate_dev:***@localhost:5432/redisgate_dev
echo - JWT_SECRET: ***configured***
echo - APP_PORT: 3000
echo.

echo [BUILD] Compiling...
cargo build --bin redisgate --release 2>&1 | findstr /i "finished error warning"

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo [INFO] Building in debug mode...
    cargo build --bin redisgate 2>&1 | findstr /i "finished error"
)

echo.
echo [START] Running server...
echo.
echo ========================================
echo   Server starting on http://localhost:3000
echo   Fix page: http://localhost:3000/fix-401.html
echo ========================================
echo.

REM Try release build first, fallback to debug
if exist target\release\redisgate.exe (
    echo [INFO] Using release build
    target\release\redisgate.exe
) else (
    echo [INFO] Using debug build
    target\debug\redisgate.exe
)

pause

