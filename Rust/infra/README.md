# Infrastructure & Deployment

## Quick Start

### Development (Single Instance)

```bash
# Start single chat service for development
docker-compose -f compose.yml -f compose.dev.yml up -d chat-service-dev redis

# Access
# - WebSocket: ws://localhost:8084/api/ws
# - Health: http://localhost:8084/api/health
# - Redis GUI: http://localhost:8081
```

### Production (Load Balanced)

```bash
# Deploy with load balancing
./deploy-chat.sh

# Access
# - WebSocket: ws://localhost:8085/api/ws (load balanced)
# - HAProxy Stats: http://localhost:8404
```

## Architecture

### Development
```
Client → chat-service-dev:8084 → Redis
```

### Production
```
Client → HAProxy:8085 → [chat-service-1, chat-service-2, chat-service-3] → Redis Pub/Sub
```

## Services

| Service | Port | Description |
|---------|------|-------------|
| HAProxy | 8080 | Gateway load balancer |
| HAProxy | 8085 | Chat WebSocket load balancer |
| HAProxy | 8404 | HAProxy statistics |
| Gateway | 8083 | Payment gateway (internal) |
| Auth | 8081 | Authentication service |
| Core | 8082 | Core service |
| Chat | 8084 | Chat service (internal) |
| Redis | 6379 | Cache & Pub/Sub |
| Redis Insight | 5540 | Redis GUI |
| Kafka | 9092 | Message broker |
| Zookeeper | 2181 | Kafka coordination |

## Commands

### Build

```bash
# Build all services
docker-compose build

# Build specific service
docker-compose build chat-service-1

# Build without cache
docker-compose build --no-cache chat-service-1
```

### Start/Stop

```bash
# Start all services
docker-compose up -d

# Start specific services
docker-compose up -d redis chat-service-1

# Stop all services
docker-compose down

# Stop and remove volumes
docker-compose down -v
```

### Logs

```bash
# View all logs
docker-compose logs -f

# View specific service
docker-compose logs -f chat-service-1

# View multiple services
docker-compose logs -f chat-service-1 chat-service-2

# Last 100 lines
docker-compose logs --tail=100 chat-service-1
```

### Health Checks

```bash
# Check all services
docker-compose ps

# Check specific service health
curl http://localhost:8084/api/health

# Check through load balancer
curl http://localhost:8085/api/health
```

### Scaling

```bash
# Scale chat service
docker-compose up -d --scale chat-service-1=5

# Note: Need to update HAProxy config for new instances
```

### Restart

```bash
# Restart all services
docker-compose restart

# Restart specific service
docker-compose restart chat-service-1

# Rolling restart (zero downtime)
docker-compose restart chat-service-1 && sleep 10 && \
docker-compose restart chat-service-2 && sleep 10 && \
docker-compose restart chat-service-3
```

### Execute Commands

```bash
# Execute command in container
docker-compose exec chat-service-1 sh

# Run one-off command
docker-compose run --rm chat-service-1 cargo test
```

## Configuration Files

- `compose.yml` - Main production configuration
- `compose.dev.yml` - Development overrides
- `haproxy.cfg` - HAProxy load balancer configuration
- `Dockerfile.*` - Service-specific Dockerfiles

## Environment Variables

Create `.env` file in project root:

```env
# Database
DATABASE_URL=mysql://root@host.docker.internal:3306/rustdb

# Redis
REDIS_URL=redis://redis:6379

# Kafka
KAFKA_BROKERS=kafka:29092

# Services
AUTH_SERVICE_URL=http://auth-service:8081
CORE_SERVICE_URL=http://core-service:8082

# Logging
LOG_LEVEL=info
RUST_LOG=info

# API Keys
AUTH_API_KEYS=your-secure-api-key
```

## Networking

All services run in the same Docker network:

```bash
# List networks
docker network ls

# Inspect network
docker network inspect infra_default

# Connect to network
docker network connect infra_default my-container
```

## Volumes

Persistent data volumes:

- `redis-data` - Redis persistence
- `kafka-data` - Kafka logs
- `zookeeper-data` - Zookeeper data

```bash
# List volumes
docker volume ls

# Inspect volume
docker volume inspect infra_redis-data

# Remove unused volumes
docker volume prune
```

## Troubleshooting

### Service won't start

```bash
# Check logs
docker-compose logs chat-service-1

# Check if port is in use
netstat -an | grep 8084

# Restart service
docker-compose restart chat-service-1
```

### Can't connect to database

```bash
# Check database connectivity
docker-compose exec chat-service-1 ping host.docker.internal

# Check environment variables
docker-compose exec chat-service-1 env | grep DATABASE
```

### Redis connection issues

```bash
# Check Redis is running
docker-compose ps redis

# Test Redis connection
docker-compose exec redis redis-cli PING

# Check Redis logs
docker-compose logs redis
```

### WebSocket connection drops

```bash
# Check HAProxy stats
open http://localhost:8404

# Check service health
curl http://localhost:8085/api/health

# Check logs for errors
docker-compose logs -f chat-service-1 | grep -i error
```

## Monitoring

### HAProxy Stats

Access: http://localhost:8404

Shows:
- Active connections
- Request rate
- Health status
- Response times

### Redis Insight

Access: http://localhost:5540

Features:
- Key browser
- CLI
- Pub/Sub monitor
- Performance metrics

### Container Stats

```bash
# Real-time stats
docker stats

# Specific containers
docker stats chat-service-1 chat-service-2 chat-service-3
```

## Maintenance

### Update Services

```bash
# Pull latest images
docker-compose pull

# Rebuild and restart
docker-compose up -d --build
```

### Clean Up

```bash
# Remove stopped containers
docker-compose rm

# Remove unused images
docker image prune

# Remove everything
docker-compose down -v --rmi all
```

### Backup

```bash
# Backup volumes
docker run --rm -v infra_redis-data:/data -v $(pwd):/backup \
  alpine tar czf /backup/redis-backup.tar.gz /data

# Restore volumes
docker run --rm -v infra_redis-data:/data -v $(pwd):/backup \
  alpine tar xzf /backup/redis-backup.tar.gz -C /
```

## Production Deployment

See [chat-docker-deployment.md](../docs/chat-docker-deployment.md) for detailed production deployment guide.

## Development Workflow

1. Make code changes
2. Rebuild service: `docker-compose build chat-service-1`
3. Restart service: `docker-compose restart chat-service-1`
4. View logs: `docker-compose logs -f chat-service-1`
5. Test: Open `test-client.html` in browser

## CI/CD Integration

Example GitHub Actions workflow:

```yaml
name: Deploy Chat Service

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Build image
        run: |
          cd Rust/infra
          docker-compose build chat-service-1
      
      - name: Push to registry
        run: |
          docker tag infra-chat:latest registry.example.com/chat:latest
          docker push registry.example.com/chat:latest
      
      - name: Deploy
        run: |
          ssh user@server 'cd /app && docker-compose pull && docker-compose up -d'
```

## Support

For issues and questions:
- Check logs: `docker-compose logs -f`
- Check health: `curl http://localhost:8085/api/health`
- View HAProxy stats: http://localhost:8404
- Check documentation: `docs/chat-docker-deployment.md`
