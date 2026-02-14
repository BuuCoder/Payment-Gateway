# Quick Start - Prometheus + Grafana Monitoring

## Bước 1: Khởi động Stack

```bash
cd Rust/infra
docker-compose up -d
```

Đợi khoảng 30 giây để tất cả services khởi động.

## Bước 2: Kiểm tra Services

```bash
# Kiểm tra tất cả containers đang chạy
docker-compose ps

# Xem logs nếu có vấn đề
docker-compose logs prometheus
docker-compose logs grafana
```

## Bước 3: Truy cập Grafana

1. Mở browser: **http://localhost:3000**
2. Login:
   - Username: `admin`
   - Password: `admin`
3. (Tùy chọn) Đổi password khi được yêu cầu

## Bước 4: Xem Dashboards

Grafana đã được cấu hình sẵn 4 dashboards:

### 1. System Overview
- CPU Usage
- Memory Usage  
- Network Traffic
- Disk Usage

### 2. Containers Overview
- Container CPU Usage
- Container Memory Usage
- Container Network I/O
- Running Containers Count

### 3. Redis Overview
- Redis Status
- Memory Usage
- Connected Clients
- Total Keys
- Commands Per Second
- Cache Hit Rate

### 4. Microservices Overview
- Services Health Status
- Gateway Request Rate
- Response Time (p95, p99)
- WebSocket Connections
- Kafka Message Rate
- CPU & Memory per Service

## Bước 5: Truy cập Prometheus (Tùy chọn)

URL: **http://localhost:9090**

### Thử một số queries:

```promql
# CPU Usage
100 - (avg(irate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)

# Memory Usage
(1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes)) * 100

# Redis Commands
rate(redis_commands_processed_total[5m])

# Container Count
count(container_last_seen{name=~".+"})
```

## Bước 6: Xem Container Metrics (Tùy chọn)

URL: **http://localhost:8888**

cAdvisor cung cấp real-time metrics cho từng container.

## Ports Tóm Tắt

| Service | Port | URL |
|---------|------|-----|
| Grafana | 3000 | http://localhost:3000 |
| Prometheus | 9090 | http://localhost:9090 |
| cAdvisor | 8888 | http://localhost:8888 |
| Node Exporter | 9100 | http://localhost:9100/metrics |
| Redis Exporter | 9121 | http://localhost:9121/metrics |
| HAProxy Stats | 8404 | http://localhost:8404/stats |

## Tạo Dashboard Mới

1. Vào Grafana → Click **+** → **Dashboard**
2. Click **Add new panel**
3. Chọn **Prometheus** datasource
4. Nhập query PromQL, ví dụ:
   ```promql
   rate(http_requests_total[5m])
   ```
5. Chọn visualization (Graph, Stat, Gauge, etc.)
6. Click **Apply** → **Save dashboard**

## Custom Queries Hữu Ích

### System
```promql
# Load Average
node_load1
node_load5
node_load15

# Disk I/O
rate(node_disk_read_bytes_total[5m])
rate(node_disk_written_bytes_total[5m])

# Open File Descriptors
node_filefd_allocated
```

### Containers
```promql
# Container Restart Count
container_start_time_seconds

# Container Uptime
time() - container_start_time_seconds

# Top 5 CPU Containers
topk(5, rate(container_cpu_usage_seconds_total[5m]))

# Top 5 Memory Containers
topk(5, container_memory_usage_bytes)
```

### Redis
```promql
# Evicted Keys
rate(redis_evicted_keys_total[5m])

# Expired Keys
rate(redis_expired_keys_total[5m])

# Blocked Clients
redis_blocked_clients

# Keyspace
redis_db_keys{db="db0"}
```

## Troubleshooting

### Grafana không hiển thị data

1. Kiểm tra Prometheus datasource:
   - Configuration → Data Sources → Prometheus
   - Click **Test** button
   - Nếu failed, kiểm tra URL: `http://prometheus:9090`

2. Kiểm tra Prometheus targets:
   - Vào http://localhost:9090/targets
   - Tất cả targets phải **UP** (màu xanh)

### Metrics không có data

```bash
# Kiểm tra exporter có chạy không
curl http://localhost:9100/metrics  # Node Exporter
curl http://localhost:9121/metrics  # Redis Exporter

# Restart nếu cần
docker-compose restart node-exporter
docker-compose restart redis-exporter
```

### Container metrics không hiển thị

```bash
# cAdvisor cần privileged mode
docker-compose logs cadvisor

# Restart với privileged
docker-compose restart cadvisor
```

## Dừng Monitoring Stack

```bash
# Dừng tất cả
docker-compose down

# Dừng và xóa volumes (mất data)
docker-compose down -v
```

## Next Steps

1. Đọc [MONITORING.md](./MONITORING.md) để hiểu chi tiết hơn
2. Tạo custom dashboards cho use case của bạn
3. Thiết lập alerts cho critical metrics
4. Thêm PostgreSQL exporter nếu cần monitor database
5. Tích hợp với Alertmanager để nhận notifications

## Tài Liệu

- [Prometheus Docs](https://prometheus.io/docs/)
- [Grafana Docs](https://grafana.com/docs/)
- [PromQL Tutorial](https://prometheus.io/docs/prometheus/latest/querying/basics/)
