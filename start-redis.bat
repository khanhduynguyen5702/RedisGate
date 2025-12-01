@echo off
echo ============================================
echo   Starting Redis Server for RedisGate
echo ============================================
echo.

echo Checking if Docker is running...
docker info >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] Docker Desktop is not running!
    echo.
    echo Please start Docker Desktop first, then run this script again.
    echo.
    echo Alternatively, install Redis for Windows from:
    echo https://github.com/microsoftarchive/redis/releases
    pause
    exit /b 1
)

echo [OK] Docker is running
echo.

echo Checking if Redis container exists...
docker ps -a | findstr "redis-local" >nul 2>&1
if %errorlevel% equ 0 (
    echo Removing old Redis container...
    docker rm -f redis-local >nul 2>&1
)

echo Starting Redis container on port 6379...
docker run -d --name redis-local -p 6379:6379 redis:7-alpine

timeout /t 3 /nobreak >nul

echo.
echo Checking Redis status...
docker ps | findstr "redis-local"

echo.
echo Testing Redis connection...
docker exec redis-local redis-cli ping

echo.
echo ============================================
echo   Redis Server Started Successfully!
echo   Port: 6379
echo   Container: redis-local
echo ============================================
echo.
echo To stop Redis:  docker stop redis-local
echo To view logs:   docker logs redis-local
echo.
pause

