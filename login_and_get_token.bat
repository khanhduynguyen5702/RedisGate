@echo off
echo ========================================
echo RedisGate - Login and Get New Token
echo ========================================
echo.

REM Login with demo account
echo Step 1: Login with demo account...
curl -s -X POST http://localhost:3000/auth/login ^
  -H "Content-Type: application/json" ^
  -d "{\"email\":\"demo@redisgate.dev\",\"password\":\"demo123\"}" > login_response.json

echo.
echo Login response saved to login_response.json
echo.

REM Parse token from response (simple extraction)
echo Step 2: Extract token from response...
type login_response.json
echo.
echo.

echo ========================================
echo INSTRUCTIONS:
echo ========================================
echo 1. Open your browser to http://localhost:3000
echo 2. Clear your browser's localStorage:
echo    - Press F12 to open DevTools
echo    - Go to Console tab
echo    - Run: localStorage.clear()
echo 3. Login again with:
echo    Email: demo@redisgate.dev
echo    Password: demo123
echo 4. You will get a NEW token that works!
echo ========================================
echo.

pause

