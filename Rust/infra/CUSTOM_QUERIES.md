# Custom Prometheus Queries

## System Metrics

### CPU
```promql
# CPU Usage %
100 - (avg(irate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)

# CPU by Core
100 - (avg by (cpu) (irate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)

# Load Average
node_load1
node_load5
node_load15
```

### RAM
```promql
# RAM Usage %
(1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes)) * 100

# RAM Usage Bytes
node_memory_MemTotal_bytes - node_memory_MemAvailable_bytes

# Available RAM
node_memory_MemAvailable_bytes

# Used RAM
node_memory_MemTotal_bytes - node_memory_MemFree_bytes - node_memory_Buffers_bytes - node_memory_Cached_bytes
```

### GPU (Nếu có NVIDIA GPU Exporter)
```promql
# GPU Usage %
nvidia_gpu_duty_cycle

# GPU Memory Usage
nvidia_gpu_memory_used_bytes

# GPU Temperature
nvidia_gpu_temperature_celsius

# GPU Power Usage
nvidia_gpu_power_usage_milliwatts
```

### Disk
```promql
# Disk Usage %
(1 - (node_filesystem_avail_bytes{mountpoint="/"} / node_filesystem_size_bytes{mountpoint="/"})) * 100

# Disk Read Rate
rate(node_disk_read_bytes_total[5m])

# Disk Write Rate
rate(node_disk_written_bytes_total[5m])

# Disk IOPS
rate(node_disk_reads_completed_total[5m]) + rate(node_disk_writes_completed_total[5m])
```

### Network
```promql
# Network Receive
rate(node_network_receive_bytes_total{device!="lo"}[5m])

# Network Transmit
rate(node_network_transmit_bytes_total{device!="lo"}[5m])

# Network Errors
rate(node_network_receive_errs_total[5m])
rate(node_network_transmit_errs_total[5m])

# TCP Connections
node_netstat_Tcp_CurrEstab
```

## Redis Metrics

```promql
# Status (0=DOWN, 1=UP)
redis_up

# Memory Usage
redis_memory_used_bytes

# Memory Usage %
(redis_memory_used_bytes / redis_memory_max_bytes) * 100

# Connected Clients
redis_connected_clients

# Blocked Clients
redis_blocked_clients

# Commands Per Second
rate(redis_commands_processed_total[5m])

# Hit Rate %
redis_keyspace_hitrate * 100

# Total Keys
redis_db_keys

# Keyspace Hits
rate(redis_keyspace_hits_total[5m])

# Keyspace Misses
rate(redis_keyspace_misses_total[5m])

# Evicted Keys
rate(redis_evicted_keys_total[5m])

# Expired Keys
rate(redis_expired_keys_total[5m])

# Network Input
rate(redis_net_input_bytes_total[5m])

# Network Output
rate(redis_net_output_bytes_total[5m])
```

## Container Metrics

```promql
# Container CPU %
rate(container_cpu_usage_seconds_total{name=~".+"}[5m]) * 100

# Top 5 CPU Containers
topk(5, rate(container_cpu_usage_seconds_total{name=~".+"}[5m]) * 100)

# Container Memory
container_memory_usage_bytes{name=~".+"}

# Top 5 Memory Containers
topk(5, container_memory_usage_bytes{name=~".+"})

# Container Memory %
(container_memory_usage_bytes / container_spec_memory_limit_bytes) * 100

# Container Network RX
rate(container_network_receive_bytes_total{name=~".+"}[5m])

# Container Network TX
rate(container_network_transmit_bytes_total{name=~".+"}[5m])

# Running Containers
count(container_last_seen{name=~".+"})

# Container Uptime
time() - container_start_time_seconds{name=~".+"}
```

## WebSocket Metrics

```promql
# Active WebSocket Connections
websocket_connections{job="chat-service"}

# Total WebSocket Connections
sum(websocket_connections)

# WebSocket Connections by Instance
sum by (instance) (websocket_connections)

# WebSocket Messages Sent
rate(websocket_messages_sent_total[5m])

# WebSocket Messages Received
rate(websocket_messages_received_total[5m])

# WebSocket Errors
rate(websocket_errors_total[5m])
```

## Service Health

```promql
# Service Status (0=DOWN, 1=UP)
up{job=~".*-service"}

# Auth Service
up{job="auth-service"}

# Core Service
up{job="core-service"}

# Gateway Services
up{job="gateway"}

# Chat Services
up{job="chat-service"}

# Worker Service
up{job="worker-service"}
```

## HTTP Metrics

```promql
# Request Rate
rate(http_requests_total[5m])

# Request Rate by Service
sum by (service) (rate(http_requests_total[5m]))

# Request Rate by Status
sum by (status) (rate(http_requests_total[5m]))

# Error Rate %
(sum(rate(http_requests_total{status=~"5.."}[5m])) / sum(rate(http_requests_total[5m]))) * 100

# Success Rate %
(sum(rate(http_requests_total{status=~"2.."}[5m])) / sum(rate(http_requests_total[5m]))) * 100

# Response Time Average (ms)
rate(http_request_duration_seconds_sum[5m]) / rate(http_request_duration_seconds_count[5m]) * 1000

# Response Time p95 (ms)
histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m])) * 1000

# Response Time p99 (ms)
histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[5m])) * 1000
```

## Kafka Metrics

```promql
# Messages Produced
rate(kafka_messages_produced_total[5m])

# Messages Consumed
rate(kafka_messages_consumed_total[5m])

# Consumer Lag
kafka_consumer_lag

# Kafka Broker Status
kafka_broker_up
```

## Custom Business Metrics

```promql
# Active Chat Rooms
chat_rooms_active

# Messages Sent Rate
rate(chat_messages_sent_total[5m])

# Total Users Online
users_online_total

# Authentication Success Rate %
(sum(rate(auth_attempts_total{result="success"}[5m])) / sum(rate(auth_attempts_total[5m]))) * 100

# Failed Login Attempts
rate(auth_attempts_total{result="failed"}[5m])

# API Rate Limit Hits
rate(rate_limit_exceeded_total[5m])

# Cache Hit Ratio %
(sum(rate(cache_hits_total[5m])) / sum(rate(cache_requests_total[5m]))) * 100
```

## Aggregation Functions

```promql
# Sum
sum(metric_name)

# Average
avg(metric_name)

# Max
max(metric_name)

# Min
min(metric_name)

# Count
count(metric_name)

# Top K
topk(5, metric_name)

# Bottom K
bottomk(5, metric_name)

# Sum by Label
sum by (label_name) (metric_name)

# Average by Label
avg by (label_name) (metric_name)
```

## Time Functions

```promql
# Rate (per second)
rate(metric_name[5m])

# Increase (total increase)
increase(metric_name[5m])

# iRate (instant rate)
irate(metric_name[5m])

# Delta (difference)
delta(metric_name[5m])

# Predict Linear
predict_linear(metric_name[1h], 3600)
```

## Math Operations

```promql
# Addition
metric_a + metric_b

# Subtraction
metric_a - metric_b

# Multiplication
metric_a * 100

# Division
metric_a / metric_b

# Percentage
(metric_a / metric_b) * 100
```

## Filtering

```promql
# Exact match
metric_name{label="value"}

# Regex match
metric_name{label=~"pattern"}

# Not equal
metric_name{label!="value"}

# Regex not match
metric_name{label!~"pattern"}

# Multiple conditions
metric_name{label1="value1", label2=~"pattern"}
```

## Comparison

```promql
# Greater than
metric_name > 80

# Less than
metric_name < 20

# Equal
metric_name == 1

# Not equal
metric_name != 0

# Greater or equal
metric_name >= 50

# Less or equal
metric_name <= 100
```

## Useful Combinations

```promql
# CPU Usage > 80% for 5 minutes
avg_over_time((100 - (avg(irate(node_cpu_seconds_total{mode="idle"}[5m])) * 100))[5m:]) > 80

# Memory Usage Trend
deriv(node_memory_MemAvailable_bytes[5m])

# Disk Will Fill In (hours)
predict_linear(node_filesystem_avail_bytes[1h], 3600) / -1024/1024/1024

# Service Uptime (seconds)
time() - process_start_time_seconds

# Request Rate Change %
((rate(http_requests_total[5m]) - rate(http_requests_total[5m] offset 1h)) / rate(http_requests_total[5m] offset 1h)) * 100
```

## Sử Dụng trong Grafana

1. Vào dashboard → Add panel
2. Chọn Prometheus datasource
3. Paste query vào Query field
4. Chọn visualization type
5. Cấu hình unit, thresholds, legend
6. Click Apply

## Tips

- Sử dụng `[5m]` cho rate calculations (5 phút)
- Thêm `by (label)` để group by label
- Dùng `sum()`, `avg()` để aggregate
- Test queries trong Prometheus UI trước: http://localhost:9090
- Sử dụng `legendFormat` để custom legend: `{{label_name}}`
