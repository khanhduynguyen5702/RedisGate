@echo off
echo ========================================
echo RedisGate - Register New User
echo ========================================
echo.

echo Registering new user: test@example.com
echo.

curl -X POST http://localhost:3000/auth/register ^
  -H "Content-Type: application/json" ^
  -d "{\"email\":\"test@example.com\",\"username\":\"testuser\",\"password\":\"Password123!\",\"first_name\":\"Test\",\"last_name\":\"User\"}"

echo.
echo.
echo ========================================
echo Now login with:
echo   Email: test@example.com
echo   Password: Password123!
echo ========================================
echo.

pause

REM Now login
echo.
echo Logging in...
curl -X POST http://localhost:3000/auth/login ^
  -H "Content-Type: application/json" ^
  -d "{\"email\":\"test@example.com\",\"password\":\"Password123!\"}"

echo.
echo.
pause

