# 🔥 RedisGate - Redis Commands Quick Reference

## Base URL

```
http://localhost:3000/redis/{instance_id}
```

**Authentication**: Thêm `?_token={api_key}` vào mọi request hoặc dùng header:
```
Authorization: Bearer {api_key}
```

---

## 📝 String Operations

### PING
Test connection to Redis instance

```bash
GET /redis/{instance_id}/ping?_token={api_key}
```

**Response**:
```json
{
  "result": "PONG"
}
```

### SET
Set key to value

```bash
GET /redis/{instance_id}/set/{key}/{value}?_token={api_key}
```

**Example**:
```bash
curl "http://localhost:3000/redis/abc-123/set/username/john?_token=eyJ..."
```

**Response**:
```json
{
  "result": "OK"
}
```

### GET
Get value of key

```bash
GET /redis/{instance_id}/get/{key}?_token={api_key}
```

**Example**:
```bash
curl "http://localhost:3000/redis/abc-123/get/username?_token=eyJ..."
```

**Response**:
```json
{
  "result": "john"
}
```

### DEL
Delete key

```bash
GET /redis/{instance_id}/del/{key}?_token={api_key}
```

**Response**:
```json
{
  "result": 1
}
```

### INCR ✨ NEW
Increment integer value

```bash
GET /redis/{instance_id}/incr/{key}?_token={api_key}
```

**Example**:
```bash
curl "http://localhost:3000/redis/abc-123/incr/counter?_token=eyJ..."
```

**Response**:
```json
{
  "result": 1
}
```

### DECR ✨ NEW
Decrement integer value

```bash
GET /redis/{instance_id}/decr/{key}?_token={api_key}
```

**Response**:
```json
{
  "result": -1
}
```

### EXPIRE ✨ NEW
Set key expiration in seconds

```bash
GET /redis/{instance_id}/expire/{key}/{seconds}?_token={api_key}
```

**Example**:
```bash
# Set mykey to expire in 60 seconds
curl "http://localhost:3000/redis/abc-123/expire/mykey/60?_token=eyJ..."
```

**Response**:
```json
{
  "result": 1
}
```

- `1` = expiration was set
- `0` = key does not exist

### TTL ✨ NEW
Get time to live for key

```bash
GET /redis/{instance_id}/ttl/{key}?_token={api_key}
```

**Response**:
```json
{
  "result": 45
}
```

- Positive number = seconds until expiration
- `-1` = key exists but no expiration
- `-2` = key does not exist

### EXISTS ✨ NEW
Check if key exists

```bash
GET /redis/{instance_id}/exists/{key}?_token={api_key}
```

**Response**:
```json
{
  "result": 1
}
```

- `1` = key exists
- `0` = key does not exist

---

## 📋 List Operations

### LPUSH
Push value to head of list

```bash
GET /redis/{instance_id}/lpush/{key}/{value}?_token={api_key}
```

**Example**:
```bash
curl "http://localhost:3000/redis/abc-123/lpush/tasks/task1?_token=eyJ..."
```

**Response**:
```json
{
  "result": 1
}
```

### LPOP
Pop value from head of list

```bash
GET /redis/{instance_id}/lpop/{key}?_token={api_key}
```

**Response**:
```json
{
  "result": "task1"
}
```

---

## 🗂️ Hash Operations

### HSET
Set hash field to value

```bash
GET /redis/{instance_id}/hset/{key}/{field}/{value}?_token={api_key}
```

**Example**:
```bash
curl "http://localhost:3000/redis/abc-123/hset/user:1/name/Alice?_token=eyJ..."
```

**Response**:
```json
{
  "result": 1
}
```

### HGET
Get hash field value

```bash
GET /redis/{instance_id}/hget/{key}/{field}?_token={api_key}
```

**Example**:
```bash
curl "http://localhost:3000/redis/abc-123/hget/user:1/name?_token=eyJ..."
```

**Response**:
```json
{
  "result": "Alice"
}
```

---

## 🔧 Advanced: Generic Command (POST)

Execute any Redis command via POST

```bash
POST /redis/{instance_id}/command?_token={api_key}
Content-Type: application/json

["COMMAND", "arg1", "arg2", ...]
```

**Example - MGET**:
```bash
curl -X POST "http://localhost:3000/redis/abc-123/command?_token=eyJ..." \
  -H "Content-Type: application/json" \
  -d '["MGET", "key1", "key2", "key3"]'
```

**Example - ZADD**:
```bash
curl -X POST "http://localhost:3000/redis/abc-123/command?_token=eyJ..." \
  -H "Content-Type: application/json" \
  -d '["ZADD", "leaderboard", "100", "player1"]'
```

### Supported Commands via Generic Endpoint

**String Operations**:
- `MGET`, `MSET`, `APPEND`, `STRLEN`

**List Operations**:
- `RPUSH`, `RPOP`, `LLEN`, `LRANGE`

**Hash Operations**:
- `HDEL`, `HEXISTS`, `HGETALL`, `HKEYS`, `HVALS`

**Set Operations**:
- `SADD`, `SREM`, `SISMEMBER`, `SMEMBERS`, `SCARD`

**Sorted Set Operations**:
- `ZADD`, `ZREM`, `ZSCORE`, `ZRANK`, `ZRANGE`

---

## 💡 Usage Examples

### 1. Counter System

```bash
# Initialize counter
curl "http://localhost:3000/redis/abc-123/set/page_views/0?_token=eyJ..."

# Increment on each page view
curl "http://localhost:3000/redis/abc-123/incr/page_views?_token=eyJ..."
curl "http://localhost:3000/redis/abc-123/incr/page_views?_token=eyJ..."

# Get current count
curl "http://localhost:3000/redis/abc-123/get/page_views?_token=eyJ..."
# Response: {"result": "2"}
```

### 2. Session Management with Expiration

```bash
# Create session (expires in 1 hour = 3600 seconds)
curl "http://localhost:3000/redis/abc-123/set/session:user123/active?_token=eyJ..."
curl "http://localhost:3000/redis/abc-123/expire/session:user123/3600?_token=eyJ..."

# Check session TTL
curl "http://localhost:3000/redis/abc-123/ttl/session:user123?_token=eyJ..."
# Response: {"result": 3598}

# Check if session exists
curl "http://localhost:3000/redis/abc-123/exists/session:user123?_token=eyJ..."
# Response: {"result": 1}
```

### 3. User Profile (Hash)

```bash
# Set user profile fields
curl "http://localhost:3000/redis/abc-123/hset/user:100/name/Alice?_token=eyJ..."
curl "http://localhost:3000/redis/abc-123/hset/user:100/email/alice@example.com?_token=eyJ..."
curl "http://localhost:3000/redis/abc-123/hset/user:100/age/25?_token=eyJ..."

# Get profile field
curl "http://localhost:3000/redis/abc-123/hget/user:100/name?_token=eyJ..."
# Response: {"result": "Alice"}

# Get all fields (via generic command)
curl -X POST "http://localhost:3000/redis/abc-123/command?_token=eyJ..." \
  -H "Content-Type: application/json" \
  -d '["HGETALL", "user:100"]'
```

### 4. Task Queue (List)

```bash
# Add tasks
curl "http://localhost:3000/redis/abc-123/lpush/queue:emails/send_welcome_email?_token=eyJ..."
curl "http://localhost:3000/redis/abc-123/lpush/queue:emails/send_notification?_token=eyJ..."

# Process task
curl "http://localhost:3000/redis/abc-123/lpop/queue:emails?_token=eyJ..."
# Response: {"result": "send_notification"}

# Process next task
curl "http://localhost:3000/redis/abc-123/lpop/queue:emails?_token=eyJ..."
# Response: {"result": "send_welcome_email"}
```

### 5. Rate Limiting

```bash
# Check request count for IP
curl "http://localhost:3000/redis/abc-123/get/ratelimit:192.168.1.1?_token=eyJ..."

# If doesn't exist, create with expiration
curl "http://localhost:3000/redis/abc-123/set/ratelimit:192.168.1.1/1?_token=eyJ..."
curl "http://localhost:3000/redis/abc-123/expire/ratelimit:192.168.1.1/60?_token=eyJ..."

# Otherwise increment
curl "http://localhost:3000/redis/abc-123/incr/ratelimit:192.168.1.1?_token=eyJ..."
```

---

## 🚨 Error Responses

### 401 Unauthorized
```json
{
  "error": "Missing API key"
}
```

### 404 Not Found
```json
{
  "error": "Redis instance not found"
}
```

### 500 Internal Server Error
```json
{
  "error": "Redis command failed"
}
```

---

## 📚 Resources

- **Full Documentation**: `/docs/README.md`
- **Monitoring Guide**: `/docs/MONITORING_GUIDE.md`
- **API Reference**: `/docs/API.md`

---

**✨ NEW** = Commands added today (01/12/2025)

