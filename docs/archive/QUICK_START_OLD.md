# 🚀 Quick Start - RedisGate

## ✅ Phase 1 HOÀN THÀNH - Quota Management System

### Những gì đã làm:
1. ✅ Database migrations (quota tables + triggers)
2. ✅ Backend services (QuotaService)  
3. ✅ API endpoints (GET/PUT quota)
4. ✅ Frontend UI (quota display bars)
5. ✅ Integration vào create/delete flows

---

## 🔧 FIX SERVER HANG - Connection Pool

**Vấn đề:** Server bị timeout khi xử lý requests

**Giải pháp:** Đã thêm connection pool configuration vào `src/main.rs`:
```rust
let pool = PgPoolOptions::new()
    .max_connections(3)
    .acquire_timeout(Duration::from_secs(3))
    .connect(&database_url).await?;
```

---

## 📋 CHẠY SERVER

### Cách 1: Quick Start (Recommended)
```cmd
cd K:\RedisGate
start-release.bat
```

### Cách 2: PowerShell
```powershell
cd K:\RedisGate
$env:DATABASE_URL="postgresql://redisgate_dev:redisgate_dev_password@localhost:5432/redisgate_dev"
$env:JWT_SECRET="development_jwt_secret_key"
cargo run --bin redisgate
```

### Cách 3: Check Status
```powershell
.\status.ps1
```

---

## 🌐 MỞ DASHBOARD

1. **Mở browser:** http://localhost:3000
2. **Login:**
   - Email: `demo@redisgate.dev`
   - Password: `demo123`
3. **Hard Refresh:** `Ctrl + Shift + R` (quan trọng!)

---

## 📊 INSTANCES TRONG DATABASE

Hiện có **1 instance** tên "ka":
```
Name: ka
Slug: ka-1763456584801
Org: 4c1d2dbd-8b4d-4a75-9e17-92f0c2635851
Memory: 512 MB
```

Để xem trong database:
```powershell
docker exec redisgate-postgres psql -U redisgate_dev -d redisgate_dev -c "SELECT name, slug FROM redis_instances WHERE deleted_at IS NULL;"
```

---

## 🐛 NẾU INSTANCES VẪN KHÔNG HIỂN THỊ

### 1. Clear Browser Cache
```javascript
// Trong Browser Console (F12):
localStorage.clear()
// Sau đó logout và login lại
```

### 2. Check API Manually
```javascript
// Trong Browser Console:
const token = localStorage.getItem('authToken');
const orgId = localStorage.getItem('organizationId');

fetch(`http://localhost:3000/api/organizations/${orgId}/redis-instances`, {
    headers: { 'Authorization': `Bearer ${token}` }
})
.then(r => r.json())
.then(d => console.log('Instances:', d));
```

### 3. Verify Server is Running
```powershell
# Check process
Get-Process -Name redisgate

# Test health
Invoke-WebRequest http://localhost:3000/health

# Test login
Invoke-RestMethod -Uri "http://localhost:3000/auth/login" -Method POST -ContentType "application/json" -Body '{"email":"demo@redisgate.dev","password":"demo123"}'
```

---

## 📈 QUOTA FEATURES

### Check Quota via API
```powershell
# Get token first (login)
$login = Invoke-RestMethod -Uri "http://localhost:3000/auth/login" -Method POST -ContentType "application/json" -Body '{"email":"demo@redisgate.dev","password":"demo123"}'
$token = $login.data.token
$orgId = $login.data.organization_id

# Get quota
Invoke-RestMethod -Uri "http://localhost:3000/api/organizations/$orgId/quota" -Headers @{Authorization="Bearer $token"}
```

### Dashboard UI Shows:
- ✅ Instances used / max (với progress bar)
- ✅ Memory used / max (với progress bar)
- ✅ API Keys used / max
- ✅ Warning messages khi >= 90%

---

## 🎯 TEST QUOTA LIMITS

### 1. Thử tạo instance vượt quota:
- Tạo instance thứ 6 → Sẽ bị reject với message "Maximum instances reached"

### 2. Thử tạo instance vượt memory quota:
- Tạo instance với memory lớn → Bị reject nếu vượt tổng memory allowed

### 3. Update quota limits (admin only):
```powershell
# Tăng max instances lên 10
Invoke-RestMethod -Uri "http://localhost:3000/api/organizations/$orgId/quota" -Method PUT -ContentType "application/json" -Headers @{Authorization="Bearer $token"} -Body '{"max_instances":10}'
```

---

## 📁 FILES QUAN TRỌNG

- `src/services/quota.rs` - Quota business logic
- `src/handlers/quota.rs` - API endpoints  
- `migrations/20251118000001_add_quota_system.sql` - Database schema
- `public/dashboard.html` - Frontend UI (lines 1122-1254: quota display)
- `CURRENT_STATUS.md` - Detailed status doc

---

## 🆘 TROUBLESHOOTING

### Server won't start
```powershell
# Check Docker
docker ps

# Check database
docker exec redisgate-postgres psql -U redisgate_dev -d redisgate_dev -c "\dt"

# Rebuild
cargo clean
cargo build --release
```

### API returns 401/403
- Logout and login again
- Check localStorage has authToken and organizationId

### Instances don't show
1. Hard refresh browser (Ctrl+Shift+R)
2. Clear localStorage
3. Check console for errors
4. Verify instance exists in DB

---

## ✨ NEXT STEPS (Optional)

1. **Add Organization Selector** - UI để switch giữa các orgs
2. **Quota Alerts** - Email/notification khi đạt 80%/90%
3. **Usage Analytics** - Charts hiển thị usage overtime
4. **Billing Integration** - Tự động upgrade plan

---

**Last Updated:** 2025-11-18 17:30
**Status:** ✅ Phase 1 Complete (95%)

