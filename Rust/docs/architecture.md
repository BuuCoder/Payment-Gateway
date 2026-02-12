# Microservices Architecture

## Cấu trúc

### Crates (Shared Libraries)
- `common`: Config, errors, cache, HTTP client
- `db`: Database connection pool
- `contracts`: DTOs, events (shared types)
- `authz`: JWT authentication/authorization
- `messaging`: Kafka producer/consumer

### Services
- `auth-service` (8081): Login, register, JWT
- `core-service` (8082): Business logic, user management
- `gateway` (8080): API Gateway, rate limiting, routing
- `worker-service`: Background jobs, Kafka consumers

### Infrastructure
- PostgreSQL: Database
- Redis: Cache + rate limiting
- Kafka: Event streaming
- HAProxy: Load balancing

## Communication

### Synchronous (HTTP)
```
Client → Gateway → Auth Service
              → Core Service
```

### Asynchronous (Kafka)
```
Gateway → Kafka → Worker Service
                → Email notifications
                → Analytics
```

## Khi nào dùng pattern này
- ✅ Large team (mỗi team một service)
- ✅ Different scaling needs per service
- ✅ Independent deployment
- ✅ Polyglot persistence
- ❌ Small projects (overhead cao)
- ❌ Tight coupling giữa services
