# Giải quyết lỗi 401 Unauthorized

## Nguyên nhân
Token của bạn đã hết hạn hoặc không hợp lệ. Điều này xảy ra khi:

1. **Server đã restart** - Nếu server restart mà không có JWT_SECRET được set trong environment variable, secret key sẽ thay đổi và tất cả token cũ sẽ không còn hợp lệ.

2. **Token đã hết hạn** - Token có thời hạn 24 giờ. Sau 24 giờ, bạn cần login lại.

3. **Secret key đã thay đổi** - Nếu JWT_SECRET environment variable thay đổi giữa các lần chạy server.

## Giải pháp

### Cách 1: Login lại (Đơn giản nhất)

1. Mở trình duyệt
2. Vào trang: http://localhost:3000
3. Đăng nhập lại với tài khoản của bạn
4. Token mới sẽ được tạo và lưu vào localStorage

### Cách 2: Set JWT_SECRET cố định

Để tránh vấn đề này trong tương lai, set JWT_SECRET trong file `.env`:

```bash
# Tạo file .env trong thư mục dự án
echo JWT_SECRET=my-super-secret-key-keep-this-safe-do-not-share > .env
```

Hoặc trên Windows:

```cmd
echo JWT_SECRET=my-super-secret-key-keep-this-safe-do-not-share > .env
```

### Cách 3: Set environment variable trước khi chạy server

**Windows (CMD):**
```cmd
set JWT_SECRET=my-super-secret-key-keep-this-safe-do-not-share
cargo run
```

**Windows (PowerShell):**
```powershell
$env:JWT_SECRET="my-super-secret-key-keep-this-safe-do-not-share"
cargo run
```

## Kiểm tra token hiện tại

Bạn có thể kiểm tra token trong browser:

1. Mở DevTools (F12)
2. Vào tab Application > Local Storage > http://localhost:3000
3. Tìm key `authToken`
4. Token sẽ hiển thị ở đó

## Test API với curl

```cmd
# Set token variable
set TOKEN=<your-token-here>

# Test authentication
curl -H "Authorization: Bearer %TOKEN%" http://localhost:3000/auth/me
```

## Lưu ý quan trọng

⚠️ **KHÔNG BAO GIỜ** share JWT_SECRET hoặc token của bạn với người khác!

⚠️ Trong production, luôn sử dụng JWT_SECRET mạnh và random:
```bash
# Generate strong secret
openssl rand -base64 32
```

