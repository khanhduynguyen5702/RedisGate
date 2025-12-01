# RedisGate Scripts

Các scripts tiện ích để quản lý và test RedisGate.

## Test Scripts

### test-instance-connection.bat (Windows)
Test kết nối tới Redis instances.

```cmd
scripts\test-instance-connection.bat
```

**Chức năng:**
- Kiểm tra DATABASE_URL
- Test PostgreSQL connection
- Liệt kê Redis instances
- Test Redis server connection

### test-instance-connection.sh (Linux/Mac)
Tương tự phiên bản Windows.

```bash
chmod +x scripts/test-instance-connection.sh
./scripts/test-instance-connection.sh
```

## Development Scripts

### dev-services.sh
Start development services (PostgreSQL, Redis) using Docker Compose.

```bash
./scripts/dev-services.sh
```

### minikube-dev.sh
Start Minikube for Kubernetes development.

```bash
./scripts/minikube-dev.sh
```

## Lưu ý

- Các scripts yêu cầu Docker Desktop đang chạy
- PostgreSQL và Redis được start tự động qua docker-compose
- Database migrations chạy tự động khi start server

