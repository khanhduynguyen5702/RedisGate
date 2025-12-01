@echo off
echo ========================================
echo Cleaning up port 8080...
echo ========================================
echo.

echo Checking for processes using port 8080...
for /f "tokens=5" %%a in ('netstat -ano ^| findstr :8080') do (
    echo Found process: %%a
    taskkill /F /PID %%a 2>nul
    if %errorlevel% equ 0 (
        echo ✓ Killed process %%a
    )
)

echo.
echo Checking for redisgate.exe processes...
taskkill /F /IM redisgate.exe 2>nul
if %errorlevel% equ 0 (
    echo ✓ Killed redisgate.exe
) else (
    echo No redisgate.exe process found
)

echo.
echo Waiting 2 seconds...
timeout /t 2 >nul

echo.
echo ========================================
echo Port 8080 should be free now
echo ========================================
echo.
echo You can now run: cargo run
echo.
pause

