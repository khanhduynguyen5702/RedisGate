@echo off
REM Clean startup script - kills old processes and starts fresh

echo ========================================
echo RedisGate - Clean Startup
echo ========================================

REM Kill any existing RedisGate processes
echo [1/5] Stopping existing RedisGate processes...
taskkill /F /IM redisgate.exe 2>nul
if %errorlevel% == 0 (
    echo   ✓ Stopped existing process
    timeout /t 2 /nobreak >nul
) else (
    echo   ℹ No existing process found
)

REM Check if Docker is running
echo [2/5] Checking Docker...
docker ps >nul 2>&1
if %errorlevel% neq 0 (
    echo   ✗ Docker is not running!
    echo   Please start Docker Desktop and try again.
    pause
    exit /b 1
)
echo   ✓ Docker is running

REM Start Docker services
echo [3/5] Starting database services...
docker-compose up -d --remove-orphans 2>nul
if %errorlevel% neq 0 (
    echo   Trying to restart existing containers...
    docker start redisgate-postgres redisgate-redis 2>nul
)
echo   ✓ Services started

REM Wait for services to be ready
echo [4/5] Waiting for services to be ready...
timeout /t 3 /nobreak >nul
echo   ✓ Services ready

REM Run database migrations and start server
echo [5/5] Starting RedisGate server...
echo.
echo ========================================
cargo run

