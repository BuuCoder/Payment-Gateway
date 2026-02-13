# Load Balancing (HAProxy)

## 🔍 Load Balancing là gì?

**Load Balancing** là kỹ thuật phân phối traffic đều đến nhiều server instances để tăng hiệu năng và độ tin cậy.

**Ví dụ đơn giản:**
- Giống như siêu thị có nhiều quầy thu ngân: khách hàng được phân bổ đều
- 1 quầy bị lỗi → Khách chuyển sang quầy khác
- Nhiều khách → Mở thêm quầy (scale)
- HAProxy = người điều phối khách hàng

---

## 🎯 Phục vụ vấn đề gì?

### Vấn đề với Single Instance

```
❌ Single Instance:
┌─────────┐                    ┌──────────┐
│ Client  │ ──── Request ────► │ Gateway  │
│         │                    │Instance 1│
│         │                    │          │
└─────────┘                    └──────────┘

Vấn đề:
- Instance lỗi → Toàn bộ service chết
- Traffic cao → Instance quá tải
- Deploy → Downtime (phải tắt service)
- Không scale được
```

### Giải pháp với Load Balancer

```
✅ Load Balancer + Multiple Instances:
                    ┌──────────────┐
┌─────────┐         │   HAProxy    │
│ Client  │ ──────► │Port 8080     │
└─────────┘         │Load Balancer │
                    └──────┬───────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
        ▼                  ▼                  ▼
  ┌──────────┐       ┌──────────┐       ┌──────────┐
  │ Gateway  │       │ Gateway  │       │ Gateway  │
  │Instance 1│       │Instance 2│       │Instance 3│
  │Port 8083 │       │Port 8083 │       │Port 8083 │
  └──────────┘       └──────────┘       └──────────┘

Lợi ích:
- 1 instance lỗi → Traffic chuyển sang instances khác
- Traffic cao → Phân phối đều, không quá tải
- Deploy → Zero downtime (rolling update)
- Scale dễ dàng: Thêm/bớt instances
```

---

## 🏗️ Vai trò trong Source Code

### 1. **Traffic Distribution**
```
Request 1 → HAProxy → Instance 1
Request 2 → HAProxy → Instance 2
Request 3 → HAProxy → Instance 3
Request 4 → HAProxy → Instance 1 (round-robin)
```

### 2. **Health Check & Auto Failover**
```
HAProxy kiểm tra health mỗi 2 giây:
- Instance 1: ✅ Healthy → Nhận traffic
- Instance 2: ✅ Healthy → Nhận traffic
- Instance 3: ❌ Unhealthy → Không nhận traffic

Khi Instance 3 recovery:
- Instance 3: ✅ Healthy → Nhận traffic lại
```

### 3. **Zero Downtime Deployment**
```
Deploy Instance 1:
1. HAProxy phát hiện Instance 1 unhealthy
2. Chuyển traffic sang Instance 2 & 3
3. Deploy Instance 1
4. Instance 1 healthy → Nhận traffic lại
5. Lặp lại cho Instance 2 & 3
→ User không bị gián đoạn!
```

---

## 📖 Cách sử dụng

### 1. Start với multiple instances

```bash
# Start 2 instances
cd Rust/infra
docker compose up -d --scale gateway=2

# Start 3 instances
docker compose up -d --scale gateway=3

# Start 5 instances
docker compose up -d --scale gateway=5
```

### 2. Request qua Load Balancer

```bash
# Tất cả requests qua HAProxy (port 8080)
curl http://localhost:8080/health
curl http://localhost:8080/api/v1/payments

# HAProxy tự động phân phối đến instances
```

### 3. Stats Dashboard

**Truy cập:** http://localhost:8404/stats

**Hiển thị:**
- Số requests per instance
- Response time
- Health check status
- Error rate
- Active connections

### 4. Scale up/down

```bash
# Scale to 3 instances
docker compose up -d --scale gateway=3

# Scale to 1 instance
docker compose up -d --scale gateway=1

# Scale to 10 instances
docker compose up -d --scale gateway=10
```

---

## ⚙️ HAProxy Configuration

### Load Balancing Algorithm

```haproxy
# Round-robin (mặc định)
balance roundrobin

# Least connections
balance leastconn

# Source IP hash (sticky sessions)
balance source
```

### Health Check

```haproxy
# Health check mỗi 2 giây
option httpchk GET /health
http-check expect status 200

# Retry 3 lần trước khi đánh dấu unhealthy
check inter 2s rise 2 fall 3
```

### Timeout Configuration

```haproxy
timeout connect 5s    # Timeout kết nối
timeout client 30s    # Timeout client
timeout server 30s    # Timeout server
```

---

## 🔄 Deployment Workflow

### Zero Downtime Deployment

```bash
# Bước 1: Build image mới
docker compose build gateway

# Bước 2: Rolling update
# HAProxy tự động phát hiện instances mới/cũ
docker compose up -d --scale gateway=3 --no-recreate

# Bước 3: Kiểm tra health
curl http://localhost:8404/stats

# Bước 4: Cleanup old containers
docker compose up -d --scale gateway=3 --remove-orphans
```

**Workflow tự động:**
```
1. Start instance mới (v2)
   ↓
2. Instance v2 healthy
   ↓
3. HAProxy thêm instance v2 vào pool
   ↓
4. Stop instance cũ (v1)
   ↓
5. HAProxy remove instance v1 khỏi pool
   ↓
6. Lặp lại cho tất cả instances
```

---

## 📊 Monitoring

### HAProxy Stats

```bash
# Xem stats qua CLI
echo "show stat" | socat stdio /var/run/haproxy.sock

# Metrics quan trọng:
# - scur: Current sessions
# - smax: Max sessions
# - rate: Session rate
# - bin/bout: Bytes in/out
# - hrsp_2xx: HTTP 2xx responses
# - hrsp_5xx: HTTP 5xx responses
```

### Health Check Logs

```bash
# Xem health check logs
docker compose logs -f haproxy | grep health

# Output:
# Health check for server gateway-1 succeeded
# Health check for server gateway-2 succeeded
# Health check for server gateway-3 failed
```

---

## 🚨 Failure Scenarios

### Scenario 1: Instance bị lỗi

```
Trước:
Instance 1: ✅ (33% traffic)
Instance 2: ✅ (33% traffic)
Instance 3: ✅ (33% traffic)

Instance 3 bị lỗi:
Instance 1: ✅ (50% traffic)
Instance 2: ✅ (50% traffic)
Instance 3: ❌ (0% traffic)

Instance 3 recovery:
Instance 1: ✅ (33% traffic)
Instance 2: ✅ (33% traffic)
Instance 3: ✅ (33% traffic)
```

### Scenario 2: HAProxy bị lỗi

```
❌ Vấn đề: HAProxy là single point of failure

✅ Giải pháp: HAProxy HA (High Availability)
- 2 HAProxy instances với Keepalived
- Virtual IP failover
- Active-Passive hoặc Active-Active
```

---

## ✅ Khi nào nên dùng?

| Tình huống | Nên dùng? | Lý do |
|------------|-----------|-------|
| Production với high traffic | ✅ Có | Phân phối traffic, tránh quá tải |
| Cần high availability | ✅ Có | Auto failover khi instance lỗi |
| Horizontal scaling | ✅ Có | Dễ dàng thêm/bớt instances |
| Zero downtime deployment | ✅ Có | Rolling update không gián đoạn |
| Development đơn giản | ❌ Không | Overhead không cần thiết |
| Low traffic applications | ❌ Không | 1 instance đủ |
| Prototype/POC | ❌ Không | Phức tạp không cần thiết |

---

## 🎯 Best Practices

### 1. Health Check Endpoint
```rust
// Implement health check endpoint
#[get("/health")]
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "gateway",
        timestamp: Utc::now(),
    })
}
```

### 2. Graceful Shutdown
```rust
// Handle SIGTERM để graceful shutdown
tokio::select! {
    _ = shutdown_signal() => {
        info!("Shutting down gracefully...");
        // Đợi requests hiện tại hoàn thành
        server.graceful_shutdown(Duration::from_secs(30)).await;
    }
}
```

### 3. Connection Draining
```haproxy
# Đợi connections hiện tại hoàn thành trước khi shutdown
option http-server-close
option forwardfor
```

---

## 💡 Tóm tắt

**Load Balancing với HAProxy** phân phối traffic đến nhiều instances:
- **Mục đích**: High availability, scalability, zero downtime deployment
- **Cách hoạt động**: HAProxy phân phối requests theo round-robin, health check tự động
- **Vai trò**: Traffic distribution, auto failover, rolling deployment
- **Kết quả**: Service luôn available, scale dễ dàng, deploy không downtime
