@echo off
cls
echo ========================================
echo KILLING PORT 8080 - Admin Mode
echo ========================================
echo.

echo Checking what's using port 8080...
powershell -Command "Get-NetTCPConnection -State Listen | Where-Object {$_.LocalPort -eq 8080} | Select-Object LocalAddress, LocalPort, OwningProcess, @{Name='ProcessName';Expression={(Get-Process -Id $_.OwningProcess).ProcessName}}"

echo.
echo Killing all processes on port 8080...

REM Kill httpd (Apache)
taskkill /F /IM httpd.exe 2>nul
if %errorlevel% equ 0 (
    echo ✓ Killed httpd.exe
)

REM Kill nginx
taskkill /F /IM nginx.exe 2>nul
if %errorlevel% equ 0 (
    echo ✓ Killed nginx.exe
)

REM Kill redisgate
taskkill /F /IM redisgate.exe 2>nul
if %errorlevel% equ 0 (
    echo ✓ Killed redisgate.exe
)

REM Kill any process using port 8080
for /f "tokens=5" %%a in ('netstat -ano ^| findstr ":8080"') do (
    echo Trying to kill PID: %%a
    taskkill /F /PID %%a 2>nul
)

echo.
echo Waiting 3 seconds...
timeout /t 3 >nul

echo.
echo Checking port 8080 again...
netstat -ano | findstr ":8080"
if %errorlevel% neq 0 (
    echo ✓ Port 8080 is now FREE!
) else (
    echo ✗ Port 8080 still in use
    echo.
    echo You may need to run this as Administrator:
    echo   Right-click CMD → Run as Administrator
    echo   Then run: .\kill-port-8080-admin.bat
)

echo.
pause

