@echo off
echo [1/3] Stopping existing server...
taskkill /F /IM redisgate.exe 2>NUL
timeout /t 2 /nobreak >NUL

echo [2/3] Building with fix for api_key_id nullable...
cargo build --release --bin redisgate
if %ERRORLEVEL% NEQ 0 (
    echo BUILD FAILED!
    pause
    exit /b 1
)

echo [3/3] Starting server...
echo.
echo ========================================
echo Server starting at http://localhost:3000
echo ========================================
echo.
start "RedisGate Server" cargo run --release --bin redisgate
timeout /t 3 /nobreak >NUL
echo Server started! Check the new window for logs.
echo.
pause

