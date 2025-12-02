# ✅ FIXED: api_key_id Nullable Error

## Lỗi ban đầu:
```
Failed to delete: Database field error: error occurred while decoding column "api_key_id": unexpected null; try decoding as an `Option`
```

## Root Cause:
- Database migration `20250914171418_make_api_key_id_nullable.sql` đã set `api_key_id` là **NULLABLE**
- Nhưng code trong `delete_redis_instance()` vẫn expect `api_key_id` **NOT NULL**
- Khi instance có `api_key_id = NULL`, code crash

## Giải pháp đã áp dụng:

### 1. Sử dụng `sqlx::query!` thay vì `sqlx::query`
**Trước (dòng 495-510):**
```rust
let redis_instance = sqlx::query(
    "SELECT api_key_id, namespace, slug FROM redis_instances..."
)
.bind(instance_id)
.bind(org_id)
.fetch_optional(&state.db_pool)
.await?;

let api_key_id: uuid::Uuid = redis_instance.try_get("api_key_id").map_err(...)?;
// ❌ Crash nếu api_key_id là NULL
```

**Sau:**
```rust
let redis_instance = sqlx::query!(
    "SELECT namespace, slug, api_key_id FROM redis_instances WHERE id = $1 AND organization_id = $2 AND deleted_at IS NULL",
    instance_id,
    org_id
)
.fetch_optional(&state.db_pool)
.await?
.ok_or_else(...)?;

let api_key_id = redis_instance.api_key_id; // ✅ Option<Uuid>
```

### 2. Chỉ deactivate API key khi nó tồn tại
**Trước (dòng 577-588):**
```rust
// Deactivate associated API key
sqlx::query!(
    "UPDATE api_keys SET is_active = false, updated_at = $1 WHERE id = $2",
    now,
    api_key_id  // ❌ Crash nếu NULL
)
.execute(&state.db_pool)
.await?;
```

**Sau:**
```rust
// Deactivate associated API key if it exists
if let Some(key_id) = api_key_id {
    sqlx::query!(
        "UPDATE api_keys SET is_active = false, updated_at = $1 WHERE id = $2",
        now,
        key_id  // ✅ Chỉ chạy khi có giá trị
    )
    .execute(&state.db_pool)
    .await?;
}
```

## Files đã sửa:
- ✅ `src/handlers/redis_instances.rs` (function `delete_redis_instance`)

## Testing:

### Bước 1: Build lại project
```bash
quick-rebuild.bat
```
HOẶC thủ công:
```bash
taskkill /F /IM redisgate.exe
cargo build --release --bin redisgate
cargo run --release --bin redisgate
```

### Bước 2: Test xóa instance
1. Mở http://localhost:3000
2. Login vào dashboard
3. Click nút **Delete** trên bất kỳ instance nào
4. ✅ **Kết quả mong đợi**: Instance được xóa thành công

### Expected Logs:
```
WARN redisgate::handlers::redis_instances: Kubernetes not available: ... Skipping K8s deletion.
INFO Successfully deleted Redis instance
```

## Giải thích kỹ thuật:

### Tại sao `api_key_id` có thể NULL?
- Migration `20250914171418_make_api_key_id_nullable.sql` cho phép NULL
- Instances được tạo trong simulation mode (không có K8s) có thể không có API key
- Instances cũ có thể đã bị xóa API key nhưng vẫn giữ instance

### Tại sao dùng `sqlx::query!` tốt hơn?
- `sqlx::query!` là **compile-time checked macro**
- Tự động generate Rust types từ database schema
- Type-safe: `api_key_id` sẽ là `Option<Uuid>` nếu column nullable
- `sqlx::query` là runtime-only, phải manual `.try_get()` và dễ lỗi

## Bonus: Các fix khác đã áp dụng cùng lúc
1. ✅ Kubernetes deletion giờ là optional (không crash khi K8s không available)
2. ✅ Kubernetes status check cũng optional (fallback về database status)

## Verification:

Kiểm tra database xem có instance nào có `api_key_id` NULL:
```sql
SELECT id, name, slug, api_key_id 
FROM redis_instances 
WHERE deleted_at IS NULL;
```

Nếu có instance với `api_key_id = NULL`, giờ vẫn xóa được bình thường! ✅

