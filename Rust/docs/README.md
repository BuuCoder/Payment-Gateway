# Documentation Index

## Core Concepts
- [Architecture](./architecture.md) - Tổng quan kiến trúc microservices
- [JWT Authentication](./jwt-auth.md) - Xác thực với JWT tokens
- [API Key Auth](./api-key-auth.md) - Backend service authentication

## Chat Service (Real-time Messaging)
- [Chat Service Overview](./CHAT-SERVICE-OVERVIEW.md) - 📘 **Tổng quan đơn giản** (Bắt đầu từ đây)
- [Chat Service API](./chat-service.md) - API documentation đầy đủ
- [Chat Architecture](./chat-architecture-diagram.md) - Sơ đồ kiến trúc chi tiết
- [Chat Security](./chat-security.md) - Bảo mật và phân quyền
- [Chat Docker Deployment](./chat-docker-deployment.md) - Triển khai Docker
- [Chat Rate Limiting](./chat-rate-limiting.md) - Giới hạn tốc độ
- [Chat IP Extraction](./chat-ip-extraction.md) - IP extraction và load balancing

## Performance & Scaling
- [Redis Cache](./redis-cache.md) - Cache layer để tăng tốc
- [Rate Limiting](./rate-limiting.md) - Token bucket algorithm
- [Load Balancing](./load-balancing.md) - HAProxy multi-instance

## Infrastructure
- [Docker Build](./docker-build.md) - Build với Cargo Chef
- [Kafka Events](./kafka-events.md) - Event streaming

## Quick Start

### Development

#### Backend Services
```bash
# Start infrastructure
docker compose -f infra/compose.yml up -d postgres redis kafka

# Run services locally
cargo run -p auth-service
cargo run -p core-service
cargo run -p gateway
```

#### Chat Service (WebSocket)
```bash
# Start Redis (required for chat)
docker compose -f infra/compose.yml up -d redis mysql

# Run chat service
cargo run -p chat-service

# Or run multiple instances for testing
cargo run -p chat-service  # Instance 1 on port 8084
# In another terminal:
SERVER_PORT=8085 cargo run -p chat-service  # Instance 2
```

#### Test Chat WebSocket
```bash
# Using websocat
websocat ws://localhost:8084/api/ws

# Or open test client in browser
open services/chat-service/test-client.html
```

### Production

#### Full Stack Deployment
```bash
# Build và start tất cả services
docker compose -f infra/compose.yml up -d --build

# Scale gateway
docker compose -f infra/compose.yml up -d --scale gateway=2

# Scale chat service (with HAProxy load balancer)
docker compose -f infra/compose.yml up -d \
  chat-service-1 chat-service-2 chat-service-3 haproxy
```

#### Chat Service Only
```bash
# Start dependencies
docker compose -f infra/compose.yml up -d redis mysql

# Start chat services with load balancer
docker compose -f infra/compose.yml up -d \
  chat-service-1 chat-service-2 chat-service-3 haproxy

# Check health
curl http://localhost:8085/api/health

# View HAProxy stats
open http://localhost:8404
```

## Ports

### Backend Services
- 8080: Gateway (load balanced)
- 8081: Auth Service
- 8082: Core Service

### Chat Service (WebSocket)
- 8084: Chat Service (internal, multiple instances)
- 8085: HAProxy Load Balancer (WebSocket entry point)
- 8404: HAProxy Stats Dashboard

### Infrastructure
- 3306: MySQL (chat data)
- 5432: PostgreSQL (user data)
- 6379: Redis (cache + pub/sub)
- 9092: Kafka (events)

## Service Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         Clients                              │
└──────────────────────────┬──────────────────────────────────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
        │ REST API         │ WebSocket        │
        ▼                  ▼                  │
┌───────────────┐  ┌───────────────┐         │
│    Gateway    │  │    HAProxy    │         │
│   Port 8080   │  │   Port 8085   │         │
└───────┬───────┘  └───────┬───────┘         │
        │                  │                  │
        ├──────────────────┼──────────────────┘
        │                  │
        ▼                  ▼
┌───────────────┐  ┌───────────────┐
│  Auth Service │  │ Chat Service  │
│   Port 8081   │  │ (3 instances) │
└───────────────┘  │   Port 8084   │
                   └───────┬───────┘
┌───────────────┐          │
│  Core Service │          │
│   Port 8082   │          │
└───────────────┘          │
        │                  │
        └──────────────────┘
                │
        ┌───────┼───────┐
        ▼       ▼       ▼
    [MySQL] [Redis] [Kafka]
```

## WebSocket Connection Example

```javascript
// Connect to chat service via HAProxy
const ws = new WebSocket('ws://localhost:8085/api/ws');

ws.onopen = () => {
  console.log('Connected to chat');
  
  // Join a room
  ws.send(JSON.stringify({
    type: 'join_room',
    room_id: 'room-uuid'
  }));
  
  // Send message
  ws.send(JSON.stringify({
    type: 'message',
    room_id: 'room-uuid',
    content: 'Hello!',
    message_type: 'text'
  }));
};

ws.onmessage = (event) => {
  const msg = JSON.parse(event.data);
  console.log('Received:', msg);
};
```

## REST API Examples

### Create Chat Room
```bash
curl -X POST http://localhost:8085/api/rooms \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "room_type": "direct",
    "member_ids": [2]
  }'
```

### Get Room Messages
```bash
curl http://localhost:8085/api/rooms/{room_id}/messages?limit=50 \
  -H "Authorizatio
- 8085: Web s
l logs -f chat-service-1 | grep ERROR
```

### Redis Monitoring
```bash
# Connect to Redis
docker compose -f infra/compose.yml exec redis redis-cli

# Monitor pub/sub
PSUBSCRIBE chat:room:*

# Check active channels
PUBSUB CHANNELS chat:room:*
```

## Testing

### Load Testing Chat Service
```bash
# Test load balancing
cd Rust
./test-load-balance.ps1  # Windows
# or
./test-load-balance.sh   # Linux/Mac

# Should show traffic distributed across instances
```

### Manual WebSocket Testing
```bash
# Install webs