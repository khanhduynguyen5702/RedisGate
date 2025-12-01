@echo off
cls
echo ========================================
echo  KILLING PORT 3000 COMPLETELY
echo ========================================
echo.

REM Kill all redisgate processes
echo [1/5] Killing redisgate.exe...
taskkill /F /IM redisgate.exe >nul 2>&1
if %errorlevel% equ 0 (
    echo       [OK] Killed redisgate.exe
) else (
    echo       [OK] No redisgate.exe running
)

REM Kill all cargo processes that might be running redisgate
echo [2/5] Killing cargo processes...
for /f "tokens=2" %%a in ('tasklist /FI "IMAGENAME eq cargo.exe" /FO LIST ^| findstr PID') do (
    taskkill /F /PID %%a >nul 2>&1
)
echo       [OK] Cargo processes cleaned

REM Find and kill anything using port 3000
echo [3/5] Freeing port 3000...
for /f "tokens=5" %%a in ('netstat -aon ^| findstr ":3000" ^| findstr "LISTENING"') do (
    echo       Killing PID %%a...
    taskkill /F /PID %%a >nul 2>&1
)
echo       [OK] Port 3000 freed

REM Wait for processes to fully terminate
echo [4/5] Waiting for cleanup...
timeout /t 3 /nobreak >nul
echo       [OK] Cleanup complete

REM Verify port is free
echo [5/5] Verifying port 3000...
netstat -ano | findstr ":3000" | findstr "LISTENING" >nul 2>&1
if %errorlevel% equ 0 (
    echo       [WARNING] Port still in use! Trying harder...
    for /f "tokens=5" %%a in ('netstat -aon ^| findstr ":3000" ^| findstr "LISTENING"') do (
        taskkill /F /PID %%a >nul 2>&1
    )
    timeout /t 2 /nobreak >nul
    netstat -ano | findstr ":3000" | findstr "LISTENING" >nul 2>&1
    if %errorlevel% equ 0 (
        echo       [ERROR] Port 3000 still blocked!
        echo       You may need to restart your computer.
    ) else (
        echo       [OK] Port 3000 is FREE!
    )
) else (
    echo       [OK] Port 3000 is FREE!
)

echo.
echo ========================================
echo  CLEANUP COMPLETE
echo ========================================
echo.

