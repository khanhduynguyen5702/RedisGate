@echo off
echo.
echo ========================================
echo   RedisGate - Token Debugger
echo ========================================
echo.
echo Starting token debugger in your browser...
echo.
echo This tool will help you:
echo   - Test if your current token is valid
echo   - Login and get a new token
echo   - Clear invalid tokens
echo.

REM Wait a moment
timeout /t 2 /nobreak >nul

REM Open browser
start http://localhost:3000/token-debug.html

echo.
echo Token Debugger opened in browser!
echo URL: http://localhost:3000/token-debug.html
echo.
echo ========================================
echo QUICK FIX INSTRUCTIONS:
echo ========================================
echo 1. Click "Test Current Token" button
echo 2. If you see 401 error:
echo    - Click "Clear All & Reset"
echo    - Click "Login & Get New Token"
echo    - Use: test@example.com / Password123!
echo 3. Done!
echo ========================================
echo.
pause

