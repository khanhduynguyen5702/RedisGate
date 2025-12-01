# Check if server is running
Write-Host "=== Checking RedisGate Server ===" -ForegroundColor Cyan

# Check process
$process = Get-Process redisgate -ErrorAction SilentlyContinue
if ($process) {
    Write-Host "OK: Process running (PID: $($process.Id))" -ForegroundColor Green
} else {
    Write-Host "FAIL: Process NOT running" -ForegroundColor Red
}

# Check port
$port = netstat -ano | Select-String "LISTENING" | Select-String ":3000"
if ($port) {
    Write-Host "OK: Port 3000 is LISTENING" -ForegroundColor Green
    Write-Host $port
} else {
    Write-Host "WARN: Port 3000 NOT listening" -ForegroundColor Yellow
}

# Try to connect
try {
    $response = Invoke-WebRequest -Uri "http://localhost:3000/health" -UseBasicParsing -TimeoutSec 2
    Write-Host "SUCCESS: Health endpoint responded: $($response.StatusCode)" -ForegroundColor Green
    Write-Host $response.Content
} catch {
    Write-Host "ERROR: Cannot connect to health endpoint" -ForegroundColor Red
    Write-Host $_.Exception.Message
}

Write-Host "`n=== Server Info ===" -ForegroundColor Cyan
netstat -ano | Select-String ":3000"

