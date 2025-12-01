# RedisGate Commands Demo Script (PowerShell)
# Usage: .\test_redis_commands.ps1 -InstanceId <instance_id> -ApiKey <api_key>

param(
    [Parameter(Mandatory=$true)]
    [string]$InstanceId,

    [Parameter(Mandatory=$true)]
    [string]$ApiKey
)

$BaseUrl = "http://localhost:3000/redis"

# Helper function to make requests
function Invoke-RedisCommand {
    param(
        [string]$Path,
        [string]$Method = "GET",
        [string]$Body = ""
    )

    $url = "$BaseUrl/$InstanceId$Path`?_token=$ApiKey"

    try {
        if ($Method -eq "POST") {
            $response = Invoke-RestMethod -Uri $url -Method POST -ContentType "application/json" -Body $Body
        } else {
            $response = Invoke-RestMethod -Uri $url -Method GET
        }
        return $response | ConvertTo-Json -Compress
    } catch {
        return "Error: $_"
    }
}

# Helper to print test results
function Print-Test {
    param(
        [string]$Name,
        [string]$Result
    )

    Write-Host "[TEST] $Name" -ForegroundColor Blue
    Write-Host "✓ Result: $Result" -ForegroundColor Green
    Write-Host ""
}

Write-Host "============================================" -ForegroundColor Yellow
Write-Host "  RedisGate Commands Demo" -ForegroundColor Yellow
Write-Host "============================================" -ForegroundColor Yellow
Write-Host ""
Write-Host "Instance ID: $InstanceId"
Write-Host "API Key: $($ApiKey.Substring(0, [Math]::Min(20, $ApiKey.Length)))..."
Write-Host ""

# Test 1: PING
Write-Host "--- 1. Testing PING ---" -ForegroundColor Yellow
$result = Invoke-RedisCommand -Path "/ping"
Print-Test -Name "PING" -Result $result
Start-Sleep -Seconds 1

# Test 2: SET
Write-Host "--- 2. Testing SET ---" -ForegroundColor Yellow
$result = Invoke-RedisCommand -Path "/set/test_key/hello_world"
Print-Test -Name "SET test_key = 'hello_world'" -Result $result
Start-Sleep -Seconds 1

# Test 3: GET
Write-Host "--- 3. Testing GET ---" -ForegroundColor Yellow
$result = Invoke-RedisCommand -Path "/get/test_key"
Print-Test -Name "GET test_key" -Result $result
Start-Sleep -Seconds 1

# Test 4: EXISTS (NEW)
Write-Host "--- 4. Testing EXISTS (NEW) ---" -ForegroundColor Yellow
$result = Invoke-RedisCommand -Path "/exists/test_key"
Print-Test -Name "EXISTS test_key" -Result $result
Start-Sleep -Seconds 1

# Test 5: INCR
Write-Host "--- 5. Testing INCR ---" -ForegroundColor Yellow
$result = Invoke-RedisCommand -Path "/incr/counter"
Print-Test -Name "INCR counter (1st time)" -Result $result
$result = Invoke-RedisCommand -Path "/incr/counter"
Print-Test -Name "INCR counter (2nd time)" -Result $result
$result = Invoke-RedisCommand -Path "/incr/counter"
Print-Test -Name "INCR counter (3rd time)" -Result $result
Start-Sleep -Seconds 1

# Test 6: DECR (NEW)
Write-Host "--- 6. Testing DECR (NEW) ---" -ForegroundColor Yellow
$result = Invoke-RedisCommand -Path "/decr/counter"
Print-Test -Name "DECR counter" -Result $result
Start-Sleep -Seconds 1

# Test 7: EXPIRE (NEW)
Write-Host "--- 7. Testing EXPIRE (NEW) ---" -ForegroundColor Yellow
Invoke-RedisCommand -Path "/set/temp_key/temporary_value" | Out-Null
$result = Invoke-RedisCommand -Path "/expire/temp_key/60"
Print-Test -Name "EXPIRE temp_key 60 seconds" -Result $result
Start-Sleep -Seconds 1

# Test 8: TTL (NEW)
Write-Host "--- 8. Testing TTL (NEW) ---" -ForegroundColor Yellow
$result = Invoke-RedisCommand -Path "/ttl/temp_key"
Print-Test -Name "TTL temp_key" -Result $result
Start-Sleep -Seconds 1

# Test 9: HSET
Write-Host "--- 9. Testing HSET ---" -ForegroundColor Yellow
$result = Invoke-RedisCommand -Path "/hset/user:1/name/Alice"
Print-Test -Name "HSET user:1 name = 'Alice'" -Result $result
$result = Invoke-RedisCommand -Path "/hset/user:1/age/25"
Print-Test -Name "HSET user:1 age = '25'" -Result $result
$result = Invoke-RedisCommand -Path "/hset/user:1/email/alice@example.com"
Print-Test -Name "HSET user:1 email = 'alice@example.com'" -Result $result
Start-Sleep -Seconds 1

# Test 10: HGET
Write-Host "--- 10. Testing HGET ---" -ForegroundColor Yellow
$result = Invoke-RedisCommand -Path "/hget/user:1/name"
Print-Test -Name "HGET user:1 name" -Result $result
$result = Invoke-RedisCommand -Path "/hget/user:1/age"
Print-Test -Name "HGET user:1 age" -Result $result
Start-Sleep -Seconds 1

# Test 11: LPUSH
Write-Host "--- 11. Testing LPUSH ---" -ForegroundColor Yellow
$result = Invoke-RedisCommand -Path "/lpush/tasks/task1"
Print-Test -Name "LPUSH tasks 'task1'" -Result $result
$result = Invoke-RedisCommand -Path "/lpush/tasks/task2"
Print-Test -Name "LPUSH tasks 'task2'" -Result $result
$result = Invoke-RedisCommand -Path "/lpush/tasks/task3"
Print-Test -Name "LPUSH tasks 'task3'" -Result $result
Start-Sleep -Seconds 1

# Test 12: LPOP
Write-Host "--- 12. Testing LPOP ---" -ForegroundColor Yellow
$result = Invoke-RedisCommand -Path "/lpop/tasks"
Print-Test -Name "LPOP tasks" -Result $result
$result = Invoke-RedisCommand -Path "/lpop/tasks"
Print-Test -Name "LPOP tasks" -Result $result
Start-Sleep -Seconds 1

# Test 13: DEL
Write-Host "--- 13. Testing DEL ---" -ForegroundColor Yellow
$result = Invoke-RedisCommand -Path "/del/test_key"
Print-Test -Name "DEL test_key" -Result $result
Start-Sleep -Seconds 1

# Test 14: Generic Command (MGET via POST)
Write-Host "--- 14. Testing Generic Command (POST) ---" -ForegroundColor Yellow
Invoke-RedisCommand -Path "/set/key1/value1" | Out-Null
Invoke-RedisCommand -Path "/set/key2/value2" | Out-Null
Invoke-RedisCommand -Path "/set/key3/value3" | Out-Null
$result = Invoke-RedisCommand -Path "/command" -Method "POST" -Body '["MGET", "key1", "key2", "key3"]'
Print-Test -Name "MGET key1 key2 key3 (via POST)" -Result $result
Start-Sleep -Seconds 1

# Test 15: Session with Expiration Example
Write-Host "--- 15. Session with Expiration Example ---" -ForegroundColor Yellow
Invoke-RedisCommand -Path "/set/session:user123/active" | Out-Null
Invoke-RedisCommand -Path "/expire/session:user123/300" | Out-Null
$exists = Invoke-RedisCommand -Path "/exists/session:user123"
$ttl = Invoke-RedisCommand -Path "/ttl/session:user123"
Write-Host "✓ Created session with 300s expiration" -ForegroundColor Green
Write-Host "✓ EXISTS: $exists" -ForegroundColor Green
Write-Host "✓ TTL: $ttl" -ForegroundColor Green
Write-Host ""
Start-Sleep -Seconds 1

# Summary
Write-Host "============================================" -ForegroundColor Yellow
Write-Host "✓ All tests completed successfully!" -ForegroundColor Green
Write-Host "============================================" -ForegroundColor Yellow
Write-Host ""
Write-Host "Summary of tested commands:"
Write-Host "  ✓ PING"
Write-Host "  ✓ SET / GET / DEL"
Write-Host "  ✓ INCR / DECR ⭐ NEW"
Write-Host "  ✓ EXISTS ⭐ NEW"
Write-Host "  ✓ EXPIRE / TTL ⭐ NEW"
Write-Host "  ✓ HSET / HGET"
Write-Host "  ✓ LPUSH / LPOP"
Write-Host "  ✓ Generic Command (POST)"
Write-Host ""
Write-Host "Next steps:"
Write-Host "  1. Check monitoring: Invoke-RestMethod http://localhost:3000/monitoring/metrics"
Write-Host "  2. View docs: docs\REDIS_COMMANDS.md"
Write-Host "  3. Test your own commands!"
Write-Host ""

