# Load Balancing (HAProxy)

## Tác dụng
- Phân phối traffic đều giữa các instances
- High availability (auto failover)
- Zero downtime deployment

## Kiến trúc
```
Client → HAProxy:8080 → Gateway-1:8083
                      → Gateway-2:8083
```

## Sử dụng

### Start với 2 instances
```bash
docker compose -f infra/compose.yml up -d --scale gateway=2
```

### Request qua load balancer
```bash
# Tất cả requests qua port 8080
curl http://localhost:8080/health
curl http://localhost:8080/api/payments
```

### Stats dashboard
http://localhost:8404/stats

### Scale up/down
```bash
# Scale to 3
docker compose -f infra/compose.yml up -d --scale gateway=3

# Scale to 1
docker compose -f infra/compose.yml up -d --scale gateway=1
```

## Khi nào dùng
- ✅ Production với high traffic
- ✅ Cần high availability
- ✅ Horizontal scaling
- ✅ Zero downtime deployment
- ❌ Development đơn giản
- ❌ Low traffic applications
