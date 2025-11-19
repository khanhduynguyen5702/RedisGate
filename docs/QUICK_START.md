# 🚀 Hướng Dẫn Chạy Nhanh - RedisGate

**Mục tiêu**: Chạy RedisGate trong vòng **5 phút** ⚡

---

## 📋 Yêu Cầu

- **Docker Desktop** - [Tải tại đây](https://www.docker.com/products/docker-desktop)
- **Rust** (1.70+) - [Cài đặt tại đây](https://rustup.rs)
- **Windows/Linux/Mac** đều được hỗ trợ

---

## ⚡ Cách 1: Chạy Tự Động (Windows)

```cmd
# Chỉ cần 1 lệnh!
.\start-clean.bat
```

Script này sẽ:
- ✅ Dừng các process cũ
- ✅ Kiểm tra Docker
- ✅ Khởi động PostgreSQL + Redis
- ✅ Chạy migrations
- ✅ Khởi động server

**Xong!** Mở trình duyệt: http://localhost:3000

---

## 🔧 Cách 2: Chạy Thủ Công

### Bước 1: Khởi động Docker Services

```bash
# Start PostgreSQL và Redis
docker-compose up -d

# Kiểm tra containers đang chạy
docker ps
```

Bạn sẽ thấy:
```
redisgate-postgres   Up   0.0.0.0:5432->5432/tcp
redisgate-redis      Up   0.0.0.0:6379->6379/tcp
```

### Bước 2: Chạy Server

```bash
# Build và run (migrations tự động chạy)
cargo run --bin redisgate
```

Đợi đến khi thấy:
```
INFO Server starting on 0.0.0.0:3000
INFO UI available at http://localhost:3000
```

### Bước 3: Truy Cập Dashboard

Mở trình duyệt: **http://localhost:3000**

---

## 👤 Đăng Nhập Demo

**Tài khoản có sẵn:**
- **Email**: `demo@redisgate.dev`
- **Password**: `Demo123456!`

**Hoặc đăng ký tài khoản mới** - tự động tạo organization!

---

## 🧪 Test Nhanh

### Test 1: PING Redis Instance

1. Login vào dashboard
2. Click **"Create Instance"**
3. Nhập tên: `test-instance`
4. Click **"Create"**
5. Trong instance list, click **"Test Connection"**
6. Sẽ thấy: `"result": "PONG"`

### Test 2: SET và GET

```bash
# Lấy token từ dashboard (F12 → Console → localStorage.getItem('authToken'))
TOKEN="your_jwt_token_here"
INSTANCE_ID="your_instance_id_here"

# SET
curl "http://localhost:3000/redis/$INSTANCE_ID/set/mykey/hello" \
  -H "Authorization: Bearer $TOKEN"

# GET
curl "http://localhost:3000/redis/$INSTANCE_ID/get/mykey" \
  -H "Authorization: Bearer $TOKEN"
```

**Kết quả**: `{"result": "hello"}`

---

## 🐛 Xử Lý Lỗi Thường Gặp

### ❌ "Address already in use"

**Nguyên nhân**: Đã có process khác chạy trên port 3000

**Giải pháp**:
```cmd
# Windows
taskkill /F /IM redisgate.exe

# Linux/Mac
pkill -f redisgate
```

Sau đó chạy lại server.

---

### ❌ "Docker is not running"

**Nguyên nhân**: Docker Desktop chưa khởi động

**Giải pháp**:
1. Mở **Docker Desktop**
2. Đợi biểu tượng cá voi màu xanh (system tray)
3. Chạy lại `docker-compose up -d`

---

### ❌ "Role redisgate_dev does not exist"

**Nguyên nhân**: Database chưa được setup

**Giải pháp**:
```bash
# Tạo lại database
docker-compose down -v
docker-compose up -d
# Đợi 5 giây
cargo run --bin redisgate
```

---

### ❌ "Cannot connect to Redis instance"

**Nguyên nhân**: Redis instance đang ở chế độ simulation

**Trạng thái**: Đây là bình thường trong development
- Instances được tạo trong database
- Kết nối thực tế cần Kubernetes
- Commands vẫn chạy được (simulation mode)

**Để fix** (optional):
- Setup Kubernetes (Minikube)
- Hoặc đợi Phase 2 để kết nối local Redis

---

### ❌ Dashboard không hiển thị instances

**Giải pháp**:
```javascript
// Mở Browser Console (F12) và chạy:
localStorage.clear()
// Reload trang và login lại
```

---

## 📊 Kiểm Tra Hệ Thống

### Check Docker:
```bash
docker ps --format "{{.Names}}: {{.Status}}"
```

**Mong đợi**:
```
redisgate-postgres: Up X minutes (healthy)
redisgate-redis: Up X minutes
```

### Check Database:
```bash
docker exec -it redisgate-postgres psql -U redisgate_dev -d redisgate_dev -c "\dt"
```

**Mong đợi**: Thấy 10-12 tables

### Check Server:
```bash
curl http://localhost:3000/health
```

**Mong đợi**: `{"status": "healthy"}`

---

## 🎯 Các Bước Tiếp Theo

Sau khi chạy thành công:

1. **Khám phá Dashboard** - Tạo instances, API keys
2. **Đọc API docs** - [API.md](API.md)
3. **Phát triển features** - [DEVELOPMENT.md](DEVELOPMENT.md)
4. **Deploy production** - [DEPLOYMENT.md](DEPLOYMENT.md)

---

## 📞 Cần Trợ Giúp?

- 📖 **Docs đầy đủ**: [docs/](../docs/)
- 🐛 **Báo lỗi**: [GitHub Issues](https://github.com/yourusername/redisgate/issues)
- 💬 **Hỏi đáp**: [TROUBLESHOOTING.md](TROUBLESHOOTING.md)

---

## ✅ Checklist Hoàn Thành

Nếu bạn thấy tất cả những điều sau, bạn đã setup thành công! 🎉

- [ ] Docker containers đang chạy
- [ ] Server khởi động không lỗi
- [ ] Dashboard mở được (http://localhost:3000)
- [ ] Login thành công
- [ ] Tạo instance được
- [ ] PING command trả về PONG

**Xin chúc mừng! Bạn đã sẵn sàng sử dụng RedisGate! 🚀**

---

**Cập nhật**: November 19, 2025  
**Version**: 0.1.0 (Phase 1)

