@echo off
SET TOKEN=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyX2lkIjoiMWYxZTA0MDgtOTUzMi00MmU3LTkzZmEtYTM4YTQ2NjgzZTAyIiwiZW1haWwiOiJ0ZXN0QGV4YW1wbGUuY29tIiwib3JnX2lkIjpudWxsLCJleHAiOjE3NjM3Nzk1NjQsImlhdCI6MTc2MzY5MzE2NH0.xqgxMIQOAq0IR_TWi78jDBwrbIzaOrrPNe_PziN3a28

echo Testing NEW token...
echo.

echo === Test 1: Get current user ===
curl -s -H "Authorization: Bearer %TOKEN%" http://localhost:3000/auth/me
echo.
echo.

echo === Test 2: List organizations ===
curl -s -H "Authorization: Bearer %TOKEN%" http://localhost:3000/api/organizations
echo.
echo.

pause

