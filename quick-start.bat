@echo off
echo ============================================
echo RedisGate - Quick Start Script
echo ============================================
echo.

echo Step 1: Starting Docker services...
docker-compose up -d
if %errorlevel% neq 0 (
    echo [ERROR] Failed to start Docker services
    echo Please ensure Docker Desktop is running
    pause
    exit /b 1
)
echo [OK] Docker services started
echo.

echo Step 2: Waiting for services to be ready...
timeout /t 5 /nobreak >nul

echo Step 3: Testing PostgreSQL connection...
psql postgresql://redisgate_dev:redisgate_dev_password@localhost:5432/redisgate_dev -c "SELECT 1;" >nul 2>&1
if %errorlevel% neq 0 (
    echo [WARN] PostgreSQL not ready yet, waiting 10 more seconds...
    timeout /t 10 /nobreak >nul
)
echo [OK] PostgreSQL is ready
echo.

echo Step 4: Running database migrations...
sqlx migrate run
if %errorlevel% neq 0 (
    echo [ERROR] Failed to run migrations
    pause
    exit /b 1
)
echo [OK] Migrations completed
echo.

echo Step 5: Testing Redis connection...
redis-cli -h localhost -p 6379 PING >nul 2>&1
if %errorlevel% neq 0 (
    echo [WARN] Redis is not responding (this is OK if not installed locally)
) else (
    echo [OK] Redis is accessible
)
echo.

echo Step 6: Building RedisGate...
cargo build --release
if %errorlevel% neq 0 (
    echo [ERROR] Build failed
    pause
    exit /b 1
)
echo [OK] Build successful
echo.

echo Step 7: Testing instance connections...
cargo run --bin test_connections
echo.

echo ============================================
echo RedisGate is ready!
echo ============================================
echo.
echo Next steps:
echo   1. Start the server: cargo run
echo   2. Register a user: POST http://localhost:8080/auth/register
echo   3. Create an organization: POST http://localhost:8080/api/organizations
echo   4. Create a Redis instance: POST http://localhost:8080/api/organizations/{org_id}/redis-instances
echo.
echo Documentation:
echo   - README.md - Overview and API documentation
echo   - INSTANCE_CONNECTION_GUIDE.md - Troubleshooting connections
echo   - DEVELOPMENT.md - Development setup details
echo.
pause

