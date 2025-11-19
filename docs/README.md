# 📚 RedisGate Documentation

Chào mừng đến với tài liệu RedisGate! Tất cả hướng dẫn bạn cần đều ở đây.

---

## 🎯 Bắt Đầu Nhanh

**Lần đầu sử dụng?** Đọc theo thứ tự:

1. **[QUICK_START.md](QUICK_START.md)** - Chạy dự án trong 5 phút
2. **[TROUBLESHOOTING.md](TROUBLESHOOTING.md)** - Nếu gặp lỗi
3. **[DEVELOPMENT.md](DEVELOPMENT.md)** - Bắt đầu phát triển

---

## 📖 Danh Mục Tài Liệu

### Cho Người Dùng:
| Tài liệu | Nội dung |
|----------|----------|
| [QUICK_START.md](QUICK_START.md) | Hướng dẫn cài đặt và chạy nhanh |
| [TROUBLESHOOTING.md](TROUBLESHOOTING.md) | Xử lý lỗi thường gặp |
| [API.md](API.md) | API endpoints _(coming soon)_ |

### Cho Developers:
| Tài liệu | Nội dung |
|----------|----------|
| [DEVELOPMENT.md](DEVELOPMENT.md) | Hướng dẫn phát triển chi tiết |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Kiến trúc hệ thống _(coming soon)_ |
| [CONTRIBUTING.md](../CONTRIBUTING.md) | Hướng dẫn đóng góp _(coming soon)_ |

### Theo Dõi Dự Án:
| Tài liệu | Nội dung |
|----------|----------|
| [STATUS.md](STATUS.md) | Trạng thái và tiến độ hiện tại |
| [CHANGELOG.md](../CHANGELOG.md) | Lịch sử thay đổi |
| [ROADMAP.md](ROADMAP.md) | Kế hoạch phát triển _(coming soon)_ |

---

## 🚀 Quick Links

### Chạy Dự Án:
```bash
# Windows
.\start-clean.bat

# Linux/Mac
docker-compose up -d && cargo run --bin redisgate
```

### Truy Cập:
- 🌐 **Dashboard**: http://localhost:3000
- 👤 **Demo**: `demo@redisgate.dev` / `Demo123456!`

### Gặp Lỗi?
→ [TROUBLESHOOTING.md](TROUBLESHOOTING.md)

---

## 📊 Trạng Thái Dự Án

**Phase 1**: 🟡 70% hoàn thành

| Component | Status |
|-----------|--------|
| Authentication | ✅ 100% |
| Organizations | ✅ 100% |
| Quota System | ✅ 100% |
| Redis Instances | 🟡 80% |
| Dashboard UI | 🟡 75% |
| Testing | ❌ 20% |

**Chi tiết**: [STATUS.md](STATUS.md)

---

## 🗂️ Cấu Trúc Thư Mục

```
docs/
├── README.md                 # ← Bạn đang ở đây
├── QUICK_START.md           # Hướng dẫn nhanh
├── DEVELOPMENT.md           # Hướng dẫn dev
├── TROUBLESHOOTING.md       # Xử lý lỗi
├── STATUS.md                # Trạng thái dự án
├── API.md                   # API docs (coming soon)
├── ARCHITECTURE.md          # Kiến trúc (coming soon)
└── archive/                 # Tài liệu cũ
    ├── PHASE1_IMPROVEMENTS.md
    ├── PHASE1_SUMMARY.md
    ├── FINAL_STATUS_REPORT.md
    └── ...
```

---

## 🎓 Học RedisGate

### Cấp Độ Beginner:
1. Đọc [../README.md](../README.md) - Tổng quan dự án
2. Chạy [QUICK_START.md](QUICK_START.md) - Setup môi trường
3. Xem demo và tạo instance đầu tiên
4. Test các Redis commands cơ bản

### Cấp Độ Intermediate:
1. Đọc [DEVELOPMENT.md](DEVELOPMENT.md)
2. Setup development environment
3. Tạo feature branch
4. Fix một issue đơn giản

### Cấp Độ Advanced:
1. Đọc architecture docs
2. Hiểu database schema
3. Contribute complex features
4. Review code của người khác

---

## 📝 Cập Nhật Tài Liệu

### Khi Thêm Feature Mới:
1. Update [CHANGELOG.md](../CHANGELOG.md)
2. Update [STATUS.md](STATUS.md)
3. Thêm vào API docs nếu có endpoint mới
4. Update DEVELOPMENT.md nếu có dev workflow mới

### Khi Fix Bug:
1. Thêm vào [TROUBLESHOOTING.md](TROUBLESHOOTING.md)
2. Update CHANGELOG.md

### Khi Thay Đổi Architecture:
1. Update ARCHITECTURE.md
2. Update diagrams
3. Thông báo cho team

---

## ❓ FAQ

### Tại sao có 2 file README?
- **[../README.md](../README.md)** - Tổng quan dự án (marketing, quick intro)
- **[docs/README.md](README.md)** - Index tài liệu chi tiết (technical)

### File nào đọc trước?
Xem phần "Bắt Đầu Nhanh" ở trên ⬆️

### Tài liệu cũ ở đâu?
→ Thư mục [archive/](archive/)

### Làm sao contribute docs?
→ Xem [../CONTRIBUTING.md](../CONTRIBUTING.md) _(coming soon)_

---

## 🔗 Liên Kết Hữu Ích

### External Docs:
- [Axum Framework](https://docs.rs/axum/latest/axum/)
- [SQLx Documentation](https://docs.rs/sqlx/latest/sqlx/)
- [Redis Commands](https://redis.io/commands/)
- [PostgreSQL Docs](https://www.postgresql.org/docs/)

### Project Links:
- [GitHub Repository](https://github.com/yourusername/redisgate)
- [Issue Tracker](https://github.com/yourusername/redisgate/issues)
- [Discussions](https://github.com/yourusername/redisgate/discussions)

---

## 📞 Hỗ Trợ

**Cần giúp đỡ?**

1. **Tìm trong docs** - Tìm kiếm trong thư mục này
2. **Check FAQ** - Phần trên có câu trả lời chưa?
3. **Troubleshooting** - [TROUBLESHOOTING.md](TROUBLESHOOTING.md)
4. **GitHub Issues** - [Tạo issue mới](https://github.com/yourusername/redisgate/issues/new)
5. **Email** - support@redisgate.io

---

## ✨ Contributing to Docs

Tài liệu luôn cần cải thiện! Nếu bạn thấy:
- ❌ Lỗi chính tả
- ❌ Thông tin lỗi thời
- ❌ Thiếu thông tin
- ✅ Cách giải thích tốt hơn

→ **Tạo Pull Request!**

### Template PR cho docs:
```markdown
## Mô tả
[Mô tả thay đổi]

## Loại thay đổi
- [ ] Sửa lỗi chính tả
- [ ] Cập nhật thông tin
- [ ] Thêm nội dung mới
- [ ] Cải thiện format

## Checklist
- [ ] Kiểm tra links
- [ ] Kiểm tra markdown format
- [ ] Test commands (nếu có)
```

---

**Happy Learning! 📚**

Last Updated: November 19, 2025

