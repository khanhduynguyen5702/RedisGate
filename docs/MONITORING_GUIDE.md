# 📊 RedisGate Monitoring Guide

## Tổng Quan

RedisGate cung cấp hệ thống monitoring và metrics đầy đủ để theo dõi hiệu suất và sức khỏe của hệ thống.

## Endpoints

### 1. Health Check

**Endpoint**: `GET /monitoring/health`

Kiểm tra xem server có đang hoạt động không.

```bash
curl http://localhost:3000/monitoring/health
```

**Response**:
```json
{
  "status": "healthy",
  "timestamp": "2025-12-01T03:30:36.162391500Z"
}
```

### 2. Readiness Check

**Endpoint**: `GET /monitoring/ready`

Kiểm tra xem server có sẵn sàng nhận requests không (bao gồm kiểm tra database).

```bash
curl http://localhost:3000/monitoring/ready
```

**Response (Ready)**:
```json
{
  "status": "ready",
  "database": "connected",
  "timestamp": "2025-12-01T03:32:54.262679100Z"
}
```

**Response (Not Ready)**:
```json
{
  "status": "not_ready",
  "database": "disconnected",
  "timestamp": "2025-12-01T03:32:54.262679100Z"
}
```

### 3. Metrics (JSON)

**Endpoint**: `GET /monitoring/metrics`

Lấy metrics hiện tại dưới dạng JSON.

```bash
curl http://localhost:3000/monitoring/metrics
```

**Response**:
```json
{
  "metrics": {
    "total_requests": 1245,
    "total_success": 1200,
    "total_errors": 45,
    "success_rate": "96.39%",
    "error_rate": "3.61%",
    "redis_commands": 890,
    "redis_connection_errors": 12,
    "auth_failures": 5,
    "uptime_seconds": 3600
  },
  "timestamp": "2025-12-01T04:00:00.000000Z"
}
```

### 4. Prometheus Metrics

**Endpoint**: `GET /monitoring/prometheus`

Lấy metrics dưới định dạng Prometheus để tích hợp với Prometheus/Grafana.

```bash
curl http://localhost:3000/monitoring/prometheus
```

**Response**:
```
# HELP redisgate_requests_total Total number of requests
# TYPE redisgate_requests_total counter
redisgate_requests_total 1245

# HELP redisgate_success_total Total number of successful responses
# TYPE redisgate_success_total counter
redisgate_success_total 1200

# HELP redisgate_errors_total Total number of errors
# TYPE redisgate_errors_total counter
redisgate_errors_total 45

# HELP redisgate_redis_commands_total Total number of Redis commands executed
# TYPE redisgate_redis_commands_total counter
redisgate_redis_commands_total 890

# HELP redisgate_redis_connection_errors_total Total number of Redis connection errors
# TYPE redisgate_redis_connection_errors_total counter
redisgate_redis_connection_errors_total 12

# HELP redisgate_auth_failures_total Total number of authentication failures
# TYPE redisgate_auth_failures_total counter
redisgate_auth_failures_total 5

# HELP redisgate_uptime_seconds Server uptime in seconds
# TYPE redisgate_uptime_seconds gauge
redisgate_uptime_seconds 3600
```

## Metrics Được Theo Dõi

| Metric | Type | Description |
|--------|------|-------------|
| `total_requests` | Counter | Tổng số HTTP requests nhận được |
| `total_success` | Counter | Số requests thành công (2xx) |
| `total_errors` | Counter | Số requests lỗi (4xx, 5xx) |
| `success_rate` | Gauge | Tỷ lệ thành công (%) |
| `error_rate` | Gauge | Tỷ lệ lỗi (%) |
| `redis_commands` | Counter | Số Redis commands đã thực thi |
| `redis_connection_errors` | Counter | Số lần kết nối Redis thất bại |
| `auth_failures` | Counter | Số lần xác thực thất bại |
| `uptime_seconds` | Gauge | Thời gian server đã chạy (giây) |

## Tích Hợp Prometheus

### 1. Cấu hình Prometheus

Thêm vào file `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'redisgate'
    scrape_interval: 15s
    static_configs:
      - targets: ['localhost:3000']
    metrics_path: '/monitoring/prometheus'
```

### 2. Khởi động Prometheus

```bash
prometheus --config.file=prometheus.yml
```

### 3. Truy cập Prometheus UI

Mở browser: `http://localhost:9090`

## Tích Hợp Grafana

### 1. Thêm Prometheus Data Source

1. Vào Grafana: `http://localhost:3001`
2. Configuration → Data Sources → Add data source
3. Chọn Prometheus
4. URL: `http://localhost:9090`
5. Click "Save & Test"

### 2. Import Dashboard

Sử dụng file `grafana/dashboard.json` có sẵn hoặc tạo dashboard mới với các queries:

**Request Rate**:
```promql
rate(redisgate_requests_total[5m])
```

**Success Rate**:
```promql
rate(redisgate_success_total[5m]) / rate(redisgate_requests_total[5m]) * 100
```

**Error Rate**:
```promql
rate(redisgate_errors_total[5m]) / rate(redisgate_requests_total[5m]) * 100
```

**Redis Commands Rate**:
```promql
rate(redisgate_redis_commands_total[5m])
```

**Uptime**:
```promql
redisgate_uptime_seconds
```

## Health Checks trong Kubernetes

### Liveness Probe

```yaml
livenessProbe:
  httpGet:
    path: /monitoring/health
    port: 3000
  initialDelaySeconds: 30
  periodSeconds: 10
  timeoutSeconds: 5
  failureThreshold: 3
```

### Readiness Probe

```yaml
readinessProbe:
  httpGet:
    path: /monitoring/ready
    port: 3000
  initialDelaySeconds: 10
  periodSeconds: 5
  timeoutSeconds: 3
  failureThreshold: 3
```

## Alerting

### Prometheus Alert Rules

Tạo file `alerts.yml`:

```yaml
groups:
  - name: redisgate
    interval: 30s
    rules:
      # High error rate
      - alert: HighErrorRate
        expr: rate(redisgate_errors_total[5m]) / rate(redisgate_requests_total[5m]) > 0.05
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value }}% over the last 5 minutes"

      # Redis connection errors
      - alert: RedisConnectionErrors
        expr: rate(redisgate_redis_connection_errors_total[5m]) > 0.1
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Redis connection errors detected"
          description: "{{ $value }} Redis connection errors per second"

      # High auth failures
      - alert: HighAuthFailures
        expr: rate(redisgate_auth_failures_total[5m]) > 1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High authentication failure rate"
          description: "{{ $value }} auth failures per second"

      # Service down
      - alert: ServiceDown
        expr: up{job="redisgate"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "RedisGate service is down"
          description: "Service has been down for more than 1 minute"
```

## Best Practices

### 1. Monitoring trong Production

- ✅ Sử dụng `/monitoring/ready` cho readiness probe
- ✅ Sử dụng `/monitoring/health` cho liveness probe
- ✅ Scrape metrics mỗi 15-30 giây
- ✅ Set up alerts cho error rate > 5%
- ✅ Monitor Redis connection errors
- ✅ Track auth failures để phát hiện attacks

### 2. Performance

- Metrics tracking sử dụng atomic operations (lock-free)
- Minimal overhead (~0.1% CPU)
- Memory footprint < 1MB

### 3. Security

- Health endpoints không cần authentication
- Metrics endpoints nên protect bằng network policies trong K8s
- Không expose sensitive data trong metrics

## Troubleshooting

### Metrics không cập nhật

**Nguyên nhân**: Middleware chưa được tích hợp

**Giải pháp**: Thêm metrics tracking vào request handlers

### Prometheus không scrape được

**Nguyên nhân**: Network policy hoặc firewall

**Giải pháp**: 
```bash
# Test endpoint
curl http://localhost:3000/monitoring/prometheus

# Check firewall
netstat -an | grep 3000
```

### Database readiness check failed

**Nguyên nhân**: Database connection pool exhausted

**Giải pháp**: Tăng `max_connections` trong config.toml

## Examples

### Monitor với cURL

```bash
# Continuous monitoring (every 5 seconds)
while true; do
  curl -s http://localhost:3000/monitoring/metrics | jq '.metrics'
  sleep 5
done
```

### Monitor với Python

```python
import requests
import time

while True:
    response = requests.get('http://localhost:3000/monitoring/metrics')
    metrics = response.json()['metrics']
    
    print(f"Requests: {metrics['total_requests']}")
    print(f"Success Rate: {metrics['success_rate']}")
    print(f"Error Rate: {metrics['error_rate']}")
    print(f"Uptime: {metrics['uptime_seconds']}s")
    print("-" * 50)
    
    time.sleep(5)
```

### Dashboard Script

```bash
#!/bin/bash
# Simple CLI dashboard

clear
while true; do
  tput cup 0 0
  echo "=== RedisGate Monitoring ==="
  echo ""
  curl -s http://localhost:3000/monitoring/metrics | jq -r '
    .metrics | 
    "Total Requests:     \(.total_requests)",
    "Success Rate:       \(.success_rate)",
    "Error Rate:         \(.error_rate)",
    "Redis Commands:     \(.redis_commands)",
    "Connection Errors:  \(.redis_connection_errors)",
    "Auth Failures:      \(.auth_failures)",
    "Uptime:             \(.uptime_seconds)s"
  '
  sleep 2
done
```

---

**Tài liệu này sẽ được cập nhật khi có thêm metrics mới.**

