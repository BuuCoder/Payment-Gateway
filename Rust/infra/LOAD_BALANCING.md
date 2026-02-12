# Load Balancing với HAProxy

## Kiến trúc

```
Client
  ↓
HAProxy (Port 8080) - Load Balancer
  ↓
├─→ Gateway Instance 1 (gateway-1:8083) - Port 8003
└─→ Gateway Instance 2 (gateway-2:8083) - Port 8004
```

## Ports

- **8080**: HAProxy load balanced endpoint (RECOMMENDED)
- **8003**: Direct access to Gateway Instance 1
- **8004**: Direct access to Gateway Instance 2
- **8404**: HAProxy stats dashboard

## Start Services

### Scale Gateway to 2 instances

```bash
cd Rust
docker compose -f infra/compose.yml up -d --scale gateway=2
```

### Check running instances

```bash
docker compose -f infra/compose.yml ps gateway
```

Output:
```
NAME                IMAGE              STATUS
infra-gateway-1     infra-gateway      Up
infra-gateway-2     infra-gateway      Up
```

## Testing

### 1. Load Balanced Requests (Recommended)

```bash
# All requests go through HAProxy (round-robin)
curl http://localhost:8080/health
curl http://localhost:8080/health
curl http://localhost:8080/health
```

HAProxy will distribute requests between gateway-1 and gateway-2.

### 2. Direct Access to Specific Instance

```bash
# Gateway Instance 1
curl http://localhost:8003/health

# Gateway Instance 2
curl http://localhost:8004/health
```

### 3. Create Payment (Load Balanced)

```bash
# Login first
TOKEN=$(curl -X POST http://localhost:8081/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"john@example.com","password":"password123"}' \
  | jq -r '.token')

# Create payment via load balancer
curl -X POST http://localhost:8080/api/v1/payments \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"amount":100.50,"currency":"USD"}'
```

## HAProxy Stats Dashboard

Open in browser: http://localhost:8404/stats

**Features:**
- Real-time request statistics
- Health check status for each backend
- Request distribution
- Response times
- Error rates

**Credentials:** No authentication required (development only)

## Load Balancing Algorithm

**Round Robin** - Requests are distributed evenly:
```
Request 1 → Gateway 1
Request 2 → Gateway 2
Request 3 → Gateway 1
Request 4 → Gateway 2
...
```

## Health Checks

HAProxy checks `/health` endpoint every 3 seconds:
- **fall 3**: Mark as down after 3 failed checks
- **rise 2**: Mark as up after 2 successful checks

If one instance fails, HAProxy automatically routes all traffic to healthy instances.

## Scaling

### Scale up to 3 instances

```bash
docker compose -f infra/compose.yml up -d --scale gateway=3
```

### Scale down to 1 instance

```bash
docker compose -f infra/compose.yml up -d --scale gateway=1
```

### Check logs from all instances

```bash
docker compose -f infra/compose.yml logs -f gateway
```

## Configuration

Edit `infra/haproxy.cfg` to customize:

- **Balance algorithm**: `roundrobin`, `leastconn`, `source`
- **Health check interval**: `inter 3s`
- **Timeouts**: `timeout connect`, `timeout client`, `timeout server`

After changes:
```bash
docker compose -f infra/compose.yml restart haproxy
```

## Production Considerations

1. **Enable authentication** for stats page
2. **Use HTTPS** with SSL termination at HAProxy
3. **Increase maxconn** based on load
4. **Monitor** HAProxy metrics
5. **Set up alerts** for backend failures

## Troubleshooting

### Gateway instance not responding

```bash
# Check instance health
curl http://localhost:8003/health
curl http://localhost:8004/health

# Check HAProxy logs
docker compose -f infra/compose.yml logs haproxy

# Check gateway logs
docker compose -f infra/compose.yml logs gateway
```

### HAProxy shows backend as DOWN

1. Check gateway health endpoint
2. Verify network connectivity
3. Check HAProxy config syntax
4. Review health check settings

## Performance Testing

### Apache Bench

```bash
# 1000 requests, 10 concurrent
ab -n 1000 -c 10 http://localhost:8080/health
```

### wrk

```bash
# 10 threads, 100 connections, 30 seconds
wrk -t10 -c100 -d30s http://localhost:8080/health
```

## Architecture Benefits

✅ **High Availability**: If one instance fails, traffic routes to healthy instances  
✅ **Horizontal Scaling**: Add more instances as load increases  
✅ **Zero Downtime Deployment**: Deploy new version while keeping old running  
✅ **Load Distribution**: Evenly distribute requests across instances  
✅ **Health Monitoring**: Automatic detection and removal of unhealthy instances  
