start-redis.bat
```

Sau đó restart server:
```cmd
cargo run --bin redisgate
```

---

**Lưu ý**: Trong production, instances sẽ kết nối tới Redis servers riêng biệt (Kubernetes pods), không phải localhost.
# RedisGate - Hướng dẫn khắc phục "simulation mode"

## Vấn đề
Khi test API Redis, bạn nhận được: `"result": "PONG (simulation mode - Redis not available)"`

Nguyên nhân: **Không có Redis server nào chạy trên localhost:6379**

## ✅ Giải pháp đã thực hiện

### 1. Đã update instances trong database
Tất cả instances đã được cấu hình để kết nối tới `127.0.0.1:6379`:
```bash
cargo run --bin check_instances
# Kết quả: All instances → 127.0.0.1:6379
```

### 2. Cần khởi động Redis Server

## 🚀 Cách khởi động Redis

### Option 1: Docker (Khuyến nghị)

**Bước 1: Khởi động Docker Desktop**
- Mở Docker Desktop application
- Đợi cho đến khi biểu tượng Docker hiển thị màu xanh

**Bước 2: Chạy Redis container**
```cmd
docker run -d --name redis-local -p 6379:6379 redis:7-alpine
```

**Bước 3: Verify Redis đang chạy**
```cmd
docker ps
docker exec redis-local redis-cli ping
# Kết quả mong đợi: PONG
```

### Option 2: Redis cho Windows

Nếu Docker không hoạt động, cài Redis trực tiếp:

1. Download Redis for Windows từ: https://github.com/microsoftarchive/redis/releases
2. Giải nén và chạy `redis-server.exe`
3. Redis sẽ chạy trên port 6379

### Option 3: WSL2 + Redis

```bash
# Trong WSL2
sudo apt update
sudo apt install redis-server
sudo service redis-server start
```

## 🧪 Test kết nối Redis

Sau khi Redis đã chạy, test:

```cmd
# Test bằng tool của project
cargo run --bin test_connections

# Hoặc test trực tiếp API
curl http://localhost:3000/redis/{instance_id}/ping?_token=YOUR_TOKEN
```

## 📝 Troubleshooting

### Lỗi "port 6379 already in use"
```cmd
# Tìm process đang dùng port
netstat -ano | findstr :6379

# Dừng container cũ
docker stop redis-local
docker rm redis-local

# Start lại
docker run -d --name redis-local -p 6379:6379 redis:7-alpine
```

### Docker Desktop không chạy
- Restart Docker Desktop
- Check logs: Docker Desktop → Settings → Troubleshoot → Restart
- Nếu vẫn lỗi: Dùng Option 2 hoặc 3

## ✨ Sau khi Redis chạy

1. **Restart RedisGate server**:
   ```cmd
   cargo run --bin redisgate
   ```

2. **Test API**:
   ```
   GET http://localhost:3000/redis/{instance_id}/ping
   Authorization: Bearer YOUR_TOKEN
   ```

3. **Kết quả mong đợi**:
   ```json
   {
     "success": true,
     "result": "PONG"
   }
   ```
   ✅ Không còn "(simulation mode)" nữa!

## 🎯 Quick Start Script

Chạy file này để auto-setup:
```cmd

