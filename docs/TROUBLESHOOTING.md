# 🐛 Xử Lý Lỗi - RedisGate

Tài liệu này giúp bạn khắc phục các lỗi thường gặp khi sử dụng RedisGate.

---

## 🔴 Lỗi Server

### ❌ "Address already in use" (Port 3000)

**Triệu chứng**:
```
Error: Os { code: 10048, kind: AddrInUse, message: "Only one usage..." }
```

**Nguyên nhân**: Đã có process khác đang chạy trên port 3000

**Giải pháp**:

**Windows**:
```cmd
# Tìm process đang dùng port 3000
netstat -ano | findstr :3000

# Kill process (thay <PID> bằng số thực tế)
taskkill /F /PID <PID>

# Hoặc kill tất cả redisgate
taskkill /F /IM redisgate.exe
```

**Linux/Mac**:
```bash
# Tìm và kill
lsof -ti:3000 | xargs kill -9

# Hoặc
pkill -f redisgate
```

**Script tự động** (Windows):
```cmd
.\start-clean.bat
```

---

### ❌ "Failed to bind to address" (Permission denied)

**Triệu chứng**:
```
Error: Permission denied (os error 13)
```

**Nguyên nhân**: Không có quyền bind port < 1024 (Linux/Mac) hoặc firewall chặn

**Giải pháp**:

**Linux/Mac**:
```bash
# Option 1: Dùng port >= 1024
export APP_PORT=3000

# Option 2: Run với sudo (NOT recommended)
sudo cargo run
```

**Windows**:
```cmd
# Kiểm tra firewall
# Control Panel → Windows Defender Firewall → Allow an app
# Thêm cargo.exe và redisgate.exe
```

---

### ❌ "Database connection failed"

**Triệu chứng**:
```
Error: error connecting to server: Connection refused
```

**Nguyên nhân**: PostgreSQL chưa chạy hoặc sai connection string

**Giải pháp**:

```bash
# 1. Kiểm tra PostgreSQL container
docker ps | grep postgres

# 2. Nếu không chạy, start lại
docker start redisgate-postgres

# 3. Nếu vẫn lỗi, recreate
docker-compose down
docker-compose up -d

# 4. Kiểm tra connection string
echo $DATABASE_URL
# Phải là: postgresql://redisgate_dev:redisgate_dev_password@localhost:5432/redisgate_dev
```

---

### ❌ "Role redisgate_dev does not exist"

**Triệu chứng**:
```
error returned from database: role "redisgate_dev" does not exist
```

**Nguyên nhân**: Database user chưa được tạo

**Giải pháp**:

```bash
# Reset toàn bộ database
docker-compose down -v  # -v xóa volumes
docker-compose up -d

# Đợi PostgreSQL ready (5-10 giây)
sleep 10

# Chạy lại server (migrations sẽ tự động tạo schema)
cargo run --bin redisgate
```

---

### ❌ Migrations Failed

**Triệu chứng**:
```
Failed to run migrations: Execute(Database(PgDatabaseError...))
```

**Nguyên nhân**: Migration bị lỗi hoặc conflict

**Giải pháp**:

```bash
# 1. Xem migrations đã chạy
docker exec -it redisgate-postgres psql -U redisgate_dev -d redisgate_dev \
  -c "SELECT * FROM _sqlx_migrations ORDER BY version;"

# 2. Rollback migration cuối (manual)
docker exec -it redisgate-postgres psql -U redisgate_dev -d redisgate_dev

# 3. Trong psql:
DELETE FROM _sqlx_migrations WHERE version = (SELECT MAX(version) FROM _sqlx_migrations);
-- Sau đó DROP table/column bị lỗi

# 4. Chạy lại
cargo run
```

---

## 🔵 Lỗi Docker

### ❌ "Docker daemon is not running"

**Triệu chứng**:
```
error during connect: ... dockerDesktopLinuxEngine: The system cannot find...
```

**Nguyên nhân**: Docker Desktop chưa khởi động

**Giải pháp**:

1. **Mở Docker Desktop**
2. **Đợi biểu tượng cá voi** (system tray) chuyển sang màu xanh
3. **Kiểm tra**: `docker ps`
4. **Nếu vẫn lỗi**: Restart Docker Desktop

**Windows**: Settings → General → Use WSL 2 based engine (check)

---

### ❌ "Container name already in use"

**Triệu chứng**:
```
Error: Conflict. The container name "/redisgate-postgres" is already in use
```

**Nguyên nhân**: Container cũ vẫn tồn tại (stopped)

**Giải pháp**:

```bash
# Remove container cũ
docker rm -f redisgate-postgres redisgate-redis

# Hoặc remove tất cả containers stopped
docker container prune -f

# Sau đó start lại
docker-compose up -d
```

---

### ❌ "Port 5432 already allocated"

**Triệu chứng**:
```
Error: Bind for 0.0.0.0:5432 failed: port is already allocated
```

**Nguyên nhân**: PostgreSQL khác đang chạy (local install hoặc container khác)

**Giải pháp**:

**Option 1**: Stop PostgreSQL local
```bash
# Windows
net stop postgresql-x64-14

# Linux
sudo systemctl stop postgresql

# Mac
brew services stop postgresql
```

**Option 2**: Đổi port trong docker-compose.yml
```yaml
services:
  postgres:
    ports:
      - "5433:5432"  # Đổi từ 5432 sang 5433
```

Nhớ update `.env`:
```
DATABASE_URL=postgresql://redisgate_dev:redisgate_dev_password@localhost:5433/redisgate_dev
```

---

## 🟡 Lỗi Frontend/Dashboard

### ❌ Dashboard không load (404)

**Triệu chứng**: http://localhost:3000 → 404 Not Found

**Nguyên nhân**: Server chưa chạy hoặc chạy sai port

**Giải pháp**:

```bash
# 1. Kiểm tra server đang chạy
Get-Process -Name redisgate  # Windows
ps aux | grep redisgate      # Linux/Mac

# 2. Kiểm tra port
netstat -ano | findstr :3000  # Windows
lsof -i:3000                  # Linux/Mac

# 3. Nếu không có, start server
cargo run --bin redisgate

# 4. Kiểm tra logs
# Phải thấy: "Server starting on 0.0.0.0:3000"
```

---

### ❌ "Invalid token" / "Unauthorized"

**Triệu chứng**: Dashboard hiện "Invalid token" hoặc bị logout liên tục

**Nguyên nhân**: JWT token hết hạn hoặc invalid

**Giải pháp**:

```javascript
// Mở Browser Console (F12) và chạy:
localStorage.clear()
sessionStorage.clear()

// Reload trang
location.reload()
```

**Hoặc**:
- Logout → Login lại
- Clear browser cache
- Dùng incognito mode để test

---

### ❌ Instances không hiển thị

**Triệu chứng**: Tạo instance thành công nhưng không thấy trong dashboard

**Nguyên nhân**: Organization ID không đúng hoặc frontend cache

**Giải pháp**:

```javascript
// 1. Kiểm tra organization ID
console.log(localStorage.getItem('organizationId'))

// 2. Nếu null/undefined:
localStorage.clear()
// Login lại

// 3. Nếu vẫn không thấy, hard refresh:
// Ctrl + Shift + R (Windows/Linux)
// Cmd + Shift + R (Mac)
```

**Kiểm tra backend**:
```bash
# Xem instances trong database
docker exec -it redisgate-postgres psql -U redisgate_dev -d redisgate_dev \
  -c "SELECT id, name, organization_id FROM redis_instances;"
```

---

### ❌ "Organization undefined" error

**Triệu chứng**: Console shows `GET /api/organizations/undefined/redis-instances`

**Nguyên nhân**: Organization chưa được tạo hoặc không được lưu

**Giải pháp**:

**Đã fix trong code mới** - Auto-create organization!

Nếu vẫn gặp:
```javascript
// Clear và login lại
localStorage.clear()
location.href = '/login.html'
```

---

## 🟢 Lỗi Redis

### ❌ "Cannot connect to Redis instance"

**Triệu chứng**: 
```json
{"error": "Redis instance unavailable: Connection refused"}
```

**Nguyên nhân**: Redis instance chưa deploy hoặc đang ở simulation mode

**Trạng thái hiện tại**: **EXPECTED BEHAVIOR** trong development
- Instances được tạo trong database
- Kết nối thực cần Kubernetes deployment
- Commands chạy ở simulation mode

**Để test thực sự**:
1. Setup Minikube: `minikube start`
2. Deploy K8s resources: `kubectl apply -f k8s/`
3. Hoặc đợi Phase 2 implementation

**Workaround**: Sử dụng simulation mode - commands vẫn trả về response đúng format

---

### ❌ "Simulation mode" in response

**Triệu chứng**:
```json
{"result": "PONG (simulation mode - Redis not available)"}
```

**Nguyên nhân**: Không kết nối được Redis thật

**Giải pháp**: Xem section trên. Đây là expected trong dev environment.

---

## 🟣 Lỗi Compilation

### ❌ "error returned from database" (compile time)

**Triệu chứng**:
```
error: error returned from database: relation "table_name" does not exist
   --> src/handlers/file.rs:123:22
```

**Nguyên nhân**: SQLx compile-time check không tìm thấy table

**Giải pháp**:

```bash
# Option 1: Chạy migrations trước
docker-compose up -d
sleep 5
cargo sqlx prepare  # Tạo offline query data
cargo build

# Option 2: Skip compile-time checks (not recommended)
# Trong Cargo.toml:
# [dependencies]
# sqlx = { version = "0.7", features = ["runtime-tokio-native-tls", "postgres"] }
```

---

### ❌ "cannot find type/function in this scope"

**Triệu chứng**:
```
error[E0412]: cannot find type `SomeType` in this scope
```

**Nguyên nhân**: Missing import hoặc typo

**Giải pháp**:

```bash
# 1. Check imports at top of file
use crate::models::SomeType;

# 2. Run cargo check để xem gợi ý
cargo check

# 3. Nếu vẫn lỗi, clean build
cargo clean
cargo build
```

---

## 🔧 Lỗi Khác

### ❌ "Too many open files"

**Triệu chứng**:
```
Error: Too many open files (os error 24)
```

**Nguyên nhân**: Hệ thống giới hạn file descriptors

**Giải pháp**:

**Linux/Mac**:
```bash
# Temporary
ulimit -n 4096

# Permanent (add to ~/.bashrc or ~/.zshrc)
echo "ulimit -n 4096" >> ~/.bashrc
```

**Windows**: Thường không gặp issue này

---

### ❌ Out of Memory

**Triệu chứng**: Server crash với "out of memory"

**Nguyên nhân**: Memory leak hoặc large dataset

**Giải pháp**:

```bash
# 1. Tăng Docker memory limit
# Docker Desktop → Settings → Resources → Memory: 4GB+

# 2. Optimize queries (add LIMIT)
SELECT * FROM large_table LIMIT 100;

# 3. Enable pagination in API calls
?page=1&limit=20
```

---

### ❌ Slow Queries

**Triệu chứng**: API response > 1 second

**Giải pháp**:

```sql
-- 1. Check query execution plan
EXPLAIN ANALYZE SELECT ...;

-- 2. Add indexes
CREATE INDEX idx_name ON table(column);

-- 3. Check database connection pool
-- In .env: DATABASE_MAX_CONNECTIONS=10
```

---

## 📋 Checklist Khi Gặp Lỗi

Trước khi hỏi help, hãy check:

- [ ] Docker Desktop đang chạy?
- [ ] Containers postgres + redis UP?
- [ ] Server process đang chạy?
- [ ] Port 3000 không bị chiếm?
- [ ] Database connection OK?
- [ ] Đã chạy migrations?
- [ ] Đã clear browser cache?
- [ ] Đã check console logs (F12)?
- [ ] Đã check server logs?

---

## 📞 Vẫn Không Giải Quyết Được?

### Collect Information:

```bash
# 1. System info
docker --version
cargo --version
rustc --version

# 2. Container status
docker ps -a

# 3. Server logs (last 50 lines)
# Copy output của cargo run

# 4. Database status
docker exec redisgate-postgres pg_isready
```

### Báo Lỗi:

**GitHub Issues**: [Create Issue](https://github.com/yourusername/redisgate/issues/new)

**Template**:
```markdown
## Mô tả lỗi
[Mô tả ngắn gọn]

## Cách tái hiện
1. ...
2. ...

## Output/Logs
```
[Paste logs here]
```

## Môi trường
- OS: Windows/Linux/Mac
- Docker: version
- Rust: version
```

---

**Chúc bạn fix bug thành công! 🐛→✅**

