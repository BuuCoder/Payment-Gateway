# Documentation Index

## Core Concepts
- [Architecture](./architecture.md) - Tổng quan kiến trúc microservices
- [JWT Authentication](./jwt-auth.md) - Xác thực với JWT tokens
- [API Key Auth](./api-key-auth.md) - Backend service authentication

## Performance & Scaling
- [Redis Cache](./redis-cache.md) - Cache layer để tăng tốc
- [Rate Limiting](./rate-limiting.md) - Token bucket algorithm
- [Load Balancing](./load-balancing.md) - HAProxy multi-instance

## Infrastructure
- [Docker Build](./docker-build.md) - Build với Cargo Chef
- [Kafka Events](./kafka-events.md) - Event streaming

## Quick Start

### Development
```bash
# Start infrastructure
docker compose -f infra/compose.yml up -d postgres redis kafka

# Run services locally
cargo run -p auth-service
cargo run -p core-service
cargo run -p gateway
```

### Production
```bash
# Build và start tất cả
docker compose -f infra/compose.yml up -d --build

# Scale gateway
docker compose -f infra/compose.yml up -d --scale gateway=2
```

## Ports
- 8080: Gateway (load balanced)
- 8081: Auth Service
- 8082: Core Service
- 8404: HAProxy Stats
- 5432: PostgreSQL
- 6379: Redis
- 9092: Kafka
