# Hướng Dẫn Sử Dụng Prometheus + Grafana

## Tổng Quan

Stack monitoring này bao gồm:
- **Prometheus**: Thu thập và lưu trữ metrics
- **Grafana**: Hiển thị dashboard và visualization
- **Node Exporter**: Metrics hệ thống (CPU, RAM, Disk, Network)
- **cAdvisor**: Metrics containers Docker
- **Redis Exporter**: Metrics Redis cache

## Khởi Động

```bash
cd Rust/infra
docker-compose up -d
```

## Truy Cập Dashboards

### Grafana
- URL: http://localhost:3000
- Username: `admin`
- Password: `admin`
- Dashboards có sẵn:
  - **System Overview**: CPU, RAM, Network, Disk
  - **Containers Overview**: Metrics từng container
  - **Redis Overview**: Cache performance

### Prometheus
- URL: http://localhost:9090
- Query metrics trực tiếp
- Xem targets và alerts

### cAdvisor
- URL: http://localhost:8888
- Xem chi tiết từng container

### HAProxy Stats
- URL: http://localhost:8404/stats
- Xem load balancer statistics

## Các Metrics Quan Trọng

### System Metrics (Node Exporter)

#### CPU Usage
```promql
100 - (avg by (instance) (irate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)
```

#### Memory Usage
```promql
(1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes)) * 100
```

#### Disk Usage
```promql
(1 - (node_filesystem_avail_bytes{mountpoint="/"} / node_filesystem_size_bytes{mountpoint="/"})) * 100
```

#### Network Traffic
```promql
rate(node_network_receive_bytes_total[5m])
rate(node_network_transmit_bytes_total[5m])
```

### Container Metrics (cAdvisor)

#### Container CPU
```promql
rate(container_cpu_usage_seconds_total{name=~".+"}[5m]) * 100
```

#### Container Memory
```promql
container_memory_usage_bytes{name=~".+"}
```

#### Container Network
```promql
rate(container_network_receive_bytes_total{name=~".+"}[5m])
rate(container_network_transmit_bytes_total{name=~".+"}[5m])
```

### Redis Metrics

#### Connected Clients
```promql
redis_connected_clients
```

#### Memory Usage
```promql
redis_memory_used_bytes
```

#### Commands Per Second
```promql
rate(redis_commands_processed_total[5m])
```

#### Cache Hit Rate
```promql
redis_keyspace_hitrate * 100
```

## Custom Queries cho Database

### Thêm PostgreSQL Exporter (Tùy chọn)

Nếu bạn muốn monitor PostgreSQL, thêm vào `compose.yml`:

```yaml
postgres-exporter:
  image: prometheuscommunity/postgres-exporter:latest
  container_name: postgres-exporter
  ports:
    - "9187:9187"
  environment:
    DATA_SOURCE_NAME: "postgresql://user:password@postgres:5432/dbname?sslmode=disable"
  depends_on:
    - postgres
  restart: unless-stopped
```

Sau đó thêm vào `prometheus.yml`:

```yaml
- job_name: 'postgres'
  static_configs:
    - targets: ['postgres-exporter:9187']
      labels:
        service: 'postgres'
        type: 'database'
```

### PostgreSQL Queries Hữu Ích

#### Active Connections
```promql
pg_stat_database_numbackends
```

#### Transaction Rate
```promql
rate(pg_stat_database_xact_commit[5m])
```

#### Database Size
```promql
pg_database_size_bytes
```

#### Slow Queries
```promql
pg_stat_statements_mean_exec_time_seconds > 1
```

## Tạo Custom Dashboard

1. Truy cập Grafana: http://localhost:3000
2. Click **+** → **Dashboard**
3. Click **Add new panel**
4. Nhập PromQL query
5. Chọn visualization type
6. Click **Apply** và **Save**

### Ví Dụ Custom Panel

**Panel: Gateway Request Rate**
```promql
rate(http_requests_total{service="gateway"}[5m])
```

**Panel: Chat Service WebSocket Connections**
```promql
websocket_connections{service="chat-service"}
```

**Panel: Kafka Message Rate**
```promql
rate(kafka_messages_total[5m])
```

## Alerts (Tùy chọn)

Tạo file `Rust/infra/alerts.yml`:

```yaml
groups:
  - name: system_alerts
    interval: 30s
    rules:
      - alert: HighCPUUsage
        expr: 100 - (avg by (instance) (irate(node_cpu_seconds_total{mode="idle"}[5m])) * 100) > 80
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High CPU usage detected"
          description: "CPU usage is above 80% for 5 minutes"

      - alert: HighMemoryUsage
        expr: (1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes)) * 100 > 85
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage detected"
          description: "Memory usage is above 85% for 5 minutes"

      - alert: RedisDown
        expr: redis_up == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Redis is down"
          description: "Redis instance is not responding"

      - alert: ContainerDown
        expr: count(container_last_seen{name=~".+"}) < 10
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Container count is low"
          description: "Less than 10 containers are running"
```

Thêm vào `prometheus.yml`:

```yaml
rule_files:
  - "alerts.yml"
```

## Troubleshooting

### Prometheus không thu thập metrics

1. Kiểm tra targets: http://localhost:9090/targets
2. Xem logs: `docker logs prometheus`
3. Kiểm tra network connectivity giữa containers

### Grafana không hiển thị data

1. Kiểm tra datasource: Configuration → Data Sources
2. Test connection với Prometheus
3. Xem query inspector trong panel

### Exporter không hoạt động

```bash
# Kiểm tra logs
docker logs node-exporter
docker logs redis-exporter
docker logs cadvisor

# Restart service
docker-compose restart node-exporter
```

## Best Practices

1. **Retention**: Prometheus mặc định giữ data 15 ngày. Điều chỉnh với `--storage.tsdb.retention.time=30d`
2. **Scrape Interval**: 15s là hợp lý cho production. Giảm xuống nếu cần real-time hơn
3. **Backup**: Backup Grafana dashboards thường xuyên
4. **Security**: Đổi password Grafana mặc định
5. **Resources**: Monitor Prometheus memory usage, có thể tăng nếu cần

## Ports Summary

| Service | Port | Description |
|---------|------|-------------|
| Grafana | 3000 | Dashboard UI |
| Prometheus | 9090 | Metrics & Query UI |
| Node Exporter | 9100 | System metrics |
| Redis Exporter | 9121 | Redis metrics |
| cAdvisor | 8888 | Container metrics UI |
| HAProxy Stats | 8404 | Load balancer stats |

## Tài Liệu Tham Khảo

- [Prometheus Documentation](https://prometheus.io/docs/)
- [Grafana Documentation](https://grafana.com/docs/)
- [PromQL Cheat Sheet](https://promlabs.com/promql-cheat-sheet/)
- [Node Exporter Metrics](https://github.com/prometheus/node_exporter)
