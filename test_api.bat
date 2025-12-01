@echo off
REM Test authentication with curl

SET TOKEN=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyX2lkIjoiY2E4ZGYyYjctNjI3My00YmFkLWE0ODAtMWRkN2RhM2IyNWY5IiwiZW1haWwiOiJkZW1vQHJlZGlzZ2F0ZS5kZXYiLCJvcmdfaWQiOiI0YzFkMmRiZC04YjRkLTRhNzUtOWUxNy05MmYwYzI2MzU4NTEiLCJleHAiOjE3MzI0MjI1MjMsImlhdCI6MTczMjMzNjEyM30.AtAR-N3GXcZip8w--GBm79jOqrYxQNlJXPmuFkQN3f8
SET ORG_ID=4c1d2dbd-8b4d-4a75-9e17-92f0c2635851

echo Testing with token: %TOKEN:~0,50%...
echo.

echo === Test 1: Health check ===
curl -s http://localhost:3000/health
echo.
echo.

echo === Test 2: Get current user (with auth) ===
curl -s -H "Authorization: Bearer %TOKEN%" http://localhost:3000/auth/me
echo.
echo.

echo === Test 3: List organizations ===
curl -s -H "Authorization: Bearer %TOKEN%" http://localhost:3000/api/organizations
echo.
echo.

echo === Test 4: List Redis instances ===
curl -s -H "Authorization: Bearer %TOKEN%" http://localhost:3000/api/organizations/%ORG_ID%/redis-instances
echo.
echo.

pause

