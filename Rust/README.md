# Microservices Architecture

## Structure

```
├── crates/              # Shared libraries
│   ├── common/          # Config, errors, utils
│   ├── db/              # Database helpers
│   ├── contracts/       # DTOs, events
│   ├── authz/           # Authorization
│   └── messaging/       # Kafka/Redis
│
├── services/            # Microservices
│   ├── auth-service/    # Authentication (port 8081)
│   ├── core-service/    # Business logic (port 8082)
│   ├── gateway/         # API Gateway (port 8080)
│   └── worker-service/  # Background jobs
│
└── infra/               # Infrastructure
    └── compose.yml      # Docker Compose
```

## Run Services

```bash
# Build all services
cargo build --workspace

# Run specific service
cargo run -p auth-service
cargo run -p core-service

# Run with Docker
cd infra
docker compose up --build
```

## APIs

- Gateway: http://localhost:8080
- Auth Service: http://localhost:8081/api/auth/health
- Core Service: http://localhost:8082/api/health
