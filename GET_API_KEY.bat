@echo off
title Get Your API Key
color 0A
cls

echo.
echo ================================================
echo   🔑 Get Your Redis API Key
echo ================================================
echo.
echo Opening API Key page...
echo.

timeout /t 2 /nobreak >nul

start http://localhost:3000/get-api-key.html

echo.
echo ================================================
echo   INSTRUCTIONS:
echo ================================================
echo.
echo The page will:
echo   1. Show your Redis API Key
echo   2. Automatically save it to browser
echo   3. Provide usage instructions
echo.
echo After that:
echo   - Go to Dashboard Settings
echo   - Your API Key is already saved
echo   - Copy Instance ID from Instances tab
echo   - Test API calls!
echo.
echo ================================================
echo.

pause

