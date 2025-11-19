# 🔧 Scripts - RedisGate

Thư mục này chứa các utility scripts được tổ chức theo mục đích sử dụng.

---

## 📁 Cấu Trúc

```
scripts/
├── windows/           # Windows utility scripts
│   ├── kill-port-*.bat
│   ├── status.ps1
│   ├── check-*.bat
│   └── create-db-role.bat
│
├── testing/           # Test scripts
│   ├── test-*.ps1
│   └── verify-redis.ps1
│
├── archive/           # Old/deprecated scripts
│   ├── START.bat
│   ├── GO.bat
│   └── ... (nhiều file cũ)
│
└── (existing scripts)
    ├── dev-services.sh
    ├── minikube-dev.sh
    └── test-instance-connection.sh
```

---

## 🚀 Scripts Chính (Ở Root)

### Windows:
- **start-clean.bat** - ⭐ Script chính để chạy dự án
- **quick-start.bat** - Alternative startup script

### Linux/Mac:
- **setup-dev.sh** - Setup development environment
- **quick-start.sh** - Quick startup for Unix systems

---

## 🪟 Windows Utilities (`windows/`)

### Port Management:
- `kill-port-3000.bat` - Kill process on port 3000
- `kill-port-8080.bat` - Kill process on port 8080
- `kill-port-8080-admin.bat` - Kill admin process on 8080

### Status & Checks:
- `status.ps1` - Check system status
- `quick-status.ps1` - Quick status check
- `check-docker.bat` - Verify Docker is running
- `check-status.bat` - Check application status

### Database:
- `create-db-role.bat` - Create database role manually

---

## 🧪 Testing Scripts (`testing/`)

### Test Flows:
- `test-complete-flow.ps1` - Full E2E test
- `test-full-flow.ps1` - Complete workflow test
- `test-end-to-end.ps1` - End to end testing
- `test-flow.ps1` - Basic flow test
- `test-fullflow.ps1` - Full flow variant

### Feature Tests:
- `test-api-key.ps1` - Test API key generation
- `test-instances.ps1` - Test instance creation
- `test-redis-with-apikey.ps1` - Test Redis with API key
- `test-all-redis-commands.ps1` - Test all Redis commands
- `test-register-login.ps1` - Test user registration/login
- `test-login-simple.ps1` - Simple login test

### Verification:
- `verify-redis.ps1` - Verify Redis connectivity
- `test-existing.ps1` - Test existing setup
- `test-final.ps1` - Final verification
- `test-complete.ps1` - Complete verification

---

## 📦 Archived Scripts (`archive/`)

**Lý do archive**: Scripts cũ, trùng lặp, hoặc không còn dùng

### Old Startup Scripts:
- `START.bat`, `GO.bat`, `run.bat` - Replaced by `start-clean.bat`
- `start-server.bat`, `start-release.bat`, `start-debug.bat` - Consolidated
- `restart.bat`, `restart-complete.bat` - Not needed

### Old Setup Scripts:
- `setup-all.bat` - Replaced by simpler setup
- `validate-setup.sh` - Not maintained

### Deprecated Features:
- `seed.bat`, `seed-demo-data.*` - Demo seeding (old approach)
- `increase-org-limits.bat` - Admin task (manual now)
- `update-html-with-api.ps1` - One-time migration script

### Debug Scripts:
- `debug-instance-notfound.ps1`
- `debug-simple.ps1`
- `final-verification.ps1`

**Note**: Giữ lại cho tham khảo, nhưng không nên dùng nữa.

---

## 🎯 Sử Dụng Thường Xuyên

### 1. Chạy Dự Án:
```bash
# Windows
.\start-clean.bat

# Linux/Mac
./quick-start.sh
```

### 2. Kill Port Bị Chiếm:
```bash
# Windows
.\scripts\windows\kill-port-3000.bat
```

### 3. Check Status:
```powershell
.\scripts\windows\status.ps1
```

### 4. Run Tests:
```powershell
# Full test
.\scripts\testing\test-complete-flow.ps1

# Quick test
.\scripts\testing\test-flow.ps1
```

---

## 🔧 Development Scripts

### Existing Production Scripts:
- **dev-services.sh** - Start development services
- **minikube-dev.sh** - Minikube setup for development
- **test-instance-connection.sh** - Test Redis instance connection

**Location**: `scripts/` (root level - production ready)

---

## 📝 Naming Convention

### Prefixes:
- `start-*` - Startup scripts
- `test-*` - Testing scripts
- `check-*` - Validation scripts
- `kill-*` - Process killing utilities
- `setup-*` - Setup/installation scripts

### Suffixes:
- `.bat` - Windows batch files
- `.ps1` - PowerShell scripts
- `.sh` - Unix shell scripts

---

## 🗑️ Cleanup Policy

### Scripts được archive khi:
- [ ] Được thay thế bởi script tốt hơn
- [ ] Không được dùng > 3 tháng
- [ ] Trùng lặp chức năng
- [ ] Deprecated features

### Scripts được xóa hoàn toàn khi:
- [ ] Đã archive > 6 tháng
- [ ] Không có tham chiếu trong docs
- [ ] Không có value lịch sử
- [ ] Confirmed không cần

---

## 💡 Best Practices

### Khi Tạo Script Mới:
1. Đặt tên rõ ràng, mô tả chức năng
2. Add vào thư mục phù hợp
3. Document trong README này
4. Add error handling
5. Test thoroughly

### Khi Sửa Script:
1. Giữ backward compatibility nếu có thể
2. Update documentation
3. Test cả old và new behavior

### Khi Archive Script:
1. Di chuyển vào `archive/`
2. Document lý do trong archive README
3. Update main README
4. Check dependencies

---

## 📞 Support

**Script không chạy?**
1. Check file permissions
2. Verify paths are correct
3. See [docs/TROUBLESHOOTING.md](../docs/TROUBLESHOOTING.md)

**Cần thêm script?**
1. Create in appropriate folder
2. Update this README
3. Submit PR

---

**Last Updated**: November 19, 2025  
**Maintained by**: Development Team

