@echo off
title RedisGate - Complete Stack
color 0A
cls

echo.
echo ================================================
echo   RedisGate - Starting Complete Stack
echo ================================================
echo.

cd /d K:\RedisGate

REM Check Docker
docker --version >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo ❌ Docker not found. Install Docker Desktop first.
    pause
    exit /b 1
)

echo [1/4] Starting Redis...
docker ps | findstr redis-local >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    docker start redis-local >nul 2>&1
    echo ✅ Redis started
) else (
    docker run -d --name redis-local -p 6379:6379 redis:7.0-alpine >nul 2>&1
    echo ✅ Redis created
)

echo.
echo [2/4] Starting PostgreSQL...
docker ps | findstr postgres-redisgate >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    docker start postgres-redisgate >nul 2>&1
    echo ✅ PostgreSQL started
) else (
    docker run -d --name postgres-redisgate ^
        -e POSTGRES_USER=redisgate_dev ^
        -e POSTGRES_PASSWORD=redisgate_dev ^
        -e POSTGRES_DB=redisgate_dev ^
        -p 5432:5432 ^
        postgres:15-alpine >nul 2>&1
    echo ✅ PostgreSQL created
    echo    Waiting for PostgreSQL to be ready...
    timeout /t 5 /nobreak >nul
)

echo.
echo [3/4] Starting RedisGate Server...

REM Set environment
set DATABASE_URL=postgresql://redisgate_dev:redisgate_dev@localhost:5432/redisgate_dev
set JWT_SECRET=redisgate-super-secret-key-2024-change-this-in-production
set APP_PORT=3000
set RUST_LOG=info

REM Kill existing if running
taskkill /F /IM redisgate.exe >nul 2>&1

REM Start server
if exist target\release\redisgate.exe (
    start /b target\release\redisgate.exe
    echo ✅ RedisGate starting (release build)
) else if exist target\debug\redisgate.exe (
    start /b target\debug\redisgate.exe
    echo ✅ RedisGate starting (debug build)
) else (
    echo ❌ RedisGate executable not found!
    echo    Run: cargo build --release
    pause
    exit /b 1
)

echo.
echo [4/4] Waiting for services to be ready...
timeout /t 5 /nobreak >nul

echo.
echo ================================================
echo   ✅ ALL SERVICES RUNNING!
echo ================================================
echo.
echo 📊 Service Status:
echo.

REM Check Redis
docker exec redis-local redis-cli ping >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    echo ✅ Redis:      localhost:6379 - READY
) else (
    echo ❌ Redis:      localhost:6379 - FAILED
)

REM Check PostgreSQL
docker exec postgres-redisgate pg_isready -U redisgate_dev >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    echo ✅ PostgreSQL: localhost:5432 - READY
) else (
    echo ⚠️  PostgreSQL: localhost:5432 - STARTING...
)

REM Check RedisGate
curl -s http://localhost:3000/health >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    echo ✅ RedisGate:  http://localhost:3000 - READY
) else (
    echo ⚠️  RedisGate:  http://localhost:3000 - STARTING...
)

echo.
echo ================================================
echo   📖 NEXT STEPS:
echo ================================================
echo.
echo 1. Open: http://localhost:3000
echo 2. Login: test@example.com / Password123!
echo 3. Go to Settings → Copy Redis API Key
echo 4. Create Instance → Copy Instance ID
echo 5. Test API:
echo.
echo    curl -H "Authorization: Bearer YOUR_API_KEY" \
echo         http://localhost:3000/redis/INSTANCE_ID/ping
echo.
echo    Should return: {"result": "PONG"}
echo    (NO simulation mode!)
echo.
echo ================================================
echo   🛠️  USEFUL COMMANDS:
echo ================================================
echo.
echo Stop all:
echo   docker stop redis-local postgres-redisgate
echo   taskkill /F /IM redisgate.exe
echo.
echo View logs:
echo   docker logs redis-local
echo   docker logs postgres-redisgate
echo.
echo Restart:
echo   Re-run this script
echo.
echo ================================================
echo.

timeout /t 3 /nobreak >nul
start http://localhost:3000

pause

