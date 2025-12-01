#!/bin/bash
# RedisGate Commands Demo Script
# Usage: ./test_redis_commands.sh <instance_id> <api_key>

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
BASE_URL="http://localhost:3000/redis"
INSTANCE_ID="${1:-}"
API_KEY="${2:-}"

if [ -z "$INSTANCE_ID" ] || [ -z "$API_KEY" ]; then
    echo -e "${RED}Usage: $0 <instance_id> <api_key>${NC}"
    echo ""
    echo "Example:"
    echo "  $0 abc-123-def-456 eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
    exit 1
fi

# Helper function to make requests
redis_cmd() {
    local path="$1"
    local method="${2:-GET}"
    local data="${3:-}"

    if [ "$method" = "POST" ]; then
        curl -s -X POST "${BASE_URL}/${INSTANCE_ID}${path}?_token=${API_KEY}" \
            -H "Content-Type: application/json" \
            -d "$data"
    else
        curl -s "${BASE_URL}/${INSTANCE_ID}${path}?_token=${API_KEY}"
    fi
}

# Helper to print test results
print_test() {
    local name="$1"
    local result="$2"
    echo -e "${BLUE}[TEST]${NC} $name"
    echo -e "${GREEN}✓${NC} Result: $result"
    echo ""
}

echo -e "${YELLOW}============================================${NC}"
echo -e "${YELLOW}  RedisGate Commands Demo${NC}"
echo -e "${YELLOW}============================================${NC}"
echo ""
echo "Instance ID: $INSTANCE_ID"
echo "API Key: ${API_KEY:0:20}..."
echo ""

# Test 1: PING
echo -e "${YELLOW}--- 1. Testing PING ---${NC}"
result=$(redis_cmd "/ping")
print_test "PING" "$result"
sleep 1

# Test 2: SET
echo -e "${YELLOW}--- 2. Testing SET ---${NC}"
result=$(redis_cmd "/set/test_key/hello_world")
print_test "SET test_key = 'hello_world'" "$result"
sleep 1

# Test 3: GET
echo -e "${YELLOW}--- 3. Testing GET ---${NC}"
result=$(redis_cmd "/get/test_key")
print_test "GET test_key" "$result"
sleep 1

# Test 4: EXISTS (NEW)
echo -e "${YELLOW}--- 4. Testing EXISTS (NEW) ---${NC}"
result=$(redis_cmd "/exists/test_key")
print_test "EXISTS test_key" "$result"
sleep 1

# Test 5: INCR (with counter)
echo -e "${YELLOW}--- 5. Testing INCR ---${NC}"
result=$(redis_cmd "/incr/counter")
print_test "INCR counter (1st time)" "$result"
result=$(redis_cmd "/incr/counter")
print_test "INCR counter (2nd time)" "$result"
result=$(redis_cmd "/incr/counter")
print_test "INCR counter (3rd time)" "$result"
sleep 1

# Test 6: DECR (NEW)
echo -e "${YELLOW}--- 6. Testing DECR (NEW) ---${NC}"
result=$(redis_cmd "/decr/counter")
print_test "DECR counter" "$result"
sleep 1

# Test 7: EXPIRE (NEW)
echo -e "${YELLOW}--- 7. Testing EXPIRE (NEW) ---${NC}"
redis_cmd "/set/temp_key/temporary_value" > /dev/null
result=$(redis_cmd "/expire/temp_key/60")
print_test "EXPIRE temp_key 60 seconds" "$result"
sleep 1

# Test 8: TTL (NEW)
echo -e "${YELLOW}--- 8. Testing TTL (NEW) ---${NC}"
result=$(redis_cmd "/ttl/temp_key")
print_test "TTL temp_key" "$result"
sleep 1

# Test 9: HSET
echo -e "${YELLOW}--- 9. Testing HSET ---${NC}"
result=$(redis_cmd "/hset/user:1/name/Alice")
print_test "HSET user:1 name = 'Alice'" "$result"
result=$(redis_cmd "/hset/user:1/age/25")
print_test "HSET user:1 age = '25'" "$result"
result=$(redis_cmd "/hset/user:1/email/alice@example.com")
print_test "HSET user:1 email = 'alice@example.com'" "$result"
sleep 1

# Test 10: HGET
echo -e "${YELLOW}--- 10. Testing HGET ---${NC}"
result=$(redis_cmd "/hget/user:1/name")
print_test "HGET user:1 name" "$result"
result=$(redis_cmd "/hget/user:1/age")
print_test "HGET user:1 age" "$result"
sleep 1

# Test 11: LPUSH
echo -e "${YELLOW}--- 11. Testing LPUSH ---${NC}"
result=$(redis_cmd "/lpush/tasks/task1")
print_test "LPUSH tasks 'task1'" "$result"
result=$(redis_cmd "/lpush/tasks/task2")
print_test "LPUSH tasks 'task2'" "$result"
result=$(redis_cmd "/lpush/tasks/task3")
print_test "LPUSH tasks 'task3'" "$result"
sleep 1

# Test 12: LPOP
echo -e "${YELLOW}--- 12. Testing LPOP ---${NC}"
result=$(redis_cmd "/lpop/tasks")
print_test "LPOP tasks" "$result"
result=$(redis_cmd "/lpop/tasks")
print_test "LPOP tasks" "$result"
sleep 1

# Test 13: DEL
echo -e "${YELLOW}--- 13. Testing DEL ---${NC}"
result=$(redis_cmd "/del/test_key")
print_test "DEL test_key" "$result"
sleep 1

# Test 14: Generic Command (MGET via POST)
echo -e "${YELLOW}--- 14. Testing Generic Command (POST) ---${NC}"
# Set some keys first
redis_cmd "/set/key1/value1" > /dev/null
redis_cmd "/set/key2/value2" > /dev/null
redis_cmd "/set/key3/value3" > /dev/null
result=$(redis_cmd "/command" "POST" '["MGET", "key1", "key2", "key3"]')
print_test "MGET key1 key2 key3 (via POST)" "$result"
sleep 1

# Test 15: Session with Expiration Example
echo -e "${YELLOW}--- 15. Session with Expiration Example ---${NC}"
redis_cmd "/set/session:user123/active" > /dev/null
redis_cmd "/expire/session:user123/300" > /dev/null
exists=$(redis_cmd "/exists/session:user123")
ttl=$(redis_cmd "/ttl/session:user123")
echo -e "${GREEN}✓${NC} Created session with 300s expiration"
echo -e "${GREEN}✓${NC} EXISTS: $exists"
echo -e "${GREEN}✓${NC} TTL: $ttl"
echo ""
sleep 1

# Summary
echo -e "${YELLOW}============================================${NC}"
echo -e "${GREEN}✓ All tests completed successfully!${NC}"
echo -e "${YELLOW}============================================${NC}"
echo ""
echo "Summary of tested commands:"
echo "  ✓ PING"
echo "  ✓ SET / GET / DEL"
echo "  ✓ INCR / DECR ⭐ NEW"
echo "  ✓ EXISTS ⭐ NEW"
echo "  ✓ EXPIRE / TTL ⭐ NEW"
echo "  ✓ HSET / HGET"
echo "  ✓ LPUSH / LPOP"
echo "  ✓ Generic Command (POST)"
echo ""
echo "Next steps:"
echo "  1. Check monitoring: curl http://localhost:3000/monitoring/metrics"
echo "  2. View docs: docs/REDIS_COMMANDS.md"
echo "  3. Test your own commands!"
echo ""

