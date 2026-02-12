# Microservices Architecture - Payment Gateway

## Structure

```
├── crates/              # Shared libraries
│   ├── common/          # Config, errors, cache, HTTP client
│   ├── db/              # Database connection pool
│   ├── contracts/       # DTOs, events (shared types)
│   ├── authz/           # JWT authentication/authorization
│   └── messaging/       # Kafka producer/consumer
│
├── services/            # Microservices
│   ├── auth-service/    # Authentication + API Key (port 8081)
│   ├── core-service/    # Business logic (port 8082)
│   ├── gateway/         # API Gateway + Rate Limiting (port 8080)
│   └── worker-service/  # Background jobs (Kafka consumers)
│
├── infra/               # Infrastructure
│   ├── compose.yml      # Docker Compose
│   ├── Dockerfile.*     # Service Dockerfiles
│   └── haproxy.cfg      # Load balancer config
│
└── docs/                # Documentation
    ├── README.md        # Documentation index
    ├── architecture.md  # System architecture
    ├── jwt-auth.md      # JWT authentication
    ├── api-key-auth.md  # API Key for backend services
    ├── rate-limiting.md # Token bucket rate limiting
    ├── redis-cache.md   # Redis caching
    ├── load-balancing.md # HAProxy load balancing
    ├── docker-build.md  # Docker build guide
    └── kafka-events.md  # Event streaming
```

## Prerequisites

### 1. Install Rust
```bash
# Windows
winget install Rustlang.Rustup

# Linux/Mac
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Install VS Code Extensions (Recommended)

**Essential:**
- `rust-analyzer` - Language server (IntelliSense, go to definition, refactoring)

**Recommended:**
- `CodeLLDB` - Debugger for Rust
- `crates` - Manage Cargo dependencies
- `Even Better TOML` - TOML syntax highlighting
- `Error Lens` - Inline error messages

**Install rust-analyzer:**
```bash
# After installing Rust, restart terminal then:
rustup component add rust-analyzer
```

**VS Code Setup:**
1. Install extensions above
2. Restart VS Code
3. Open Rust project
4. Ctrl+Click to jump to definitions
5. Hover for documentation

### 3. Setup Environment
```bash
# Copy .env.example to .env
cp .env.example .env

# Edit .env with your configuration
# Required: DATABASE_URL, REDIS_URL, STRIPE_API_KEY, AUTH_API_KEYS
```

## Run Services

### Development (Local)

```bash
# Build all services
cargo build --workspace

# Run specific service
cargo run -p auth-service    # Port 8081
cargo run -p core-service    # Port 8082
cargo run -p gateway         # Port 8083
cargo run -p worker-service  # Background worker

# Run with logs
RUST_LOG=debug cargo run -p gateway
```

### Production (Docker)

```bash
# Start infrastructure (PostgreSQL, Redis, Kafka)
docker compose -f infra/compose.yml up -d postgres redis kafka

# Build and start all services
docker compose -f infra/compose.yml up -d --build

# Scale gateway to 3 instances (load balanced)
docker compose -f infra/compose.yml up -d --scale gateway=3

# View logs
docker compose -f infra/compose.yml logs -f gateway

# Stop all services
docker compose -f infra/compose.yml down
```

## API Endpoints

### Gateway (Load Balanced)
- **Base URL**: http://localhost:8080
- **Health**: `GET /health`
- **Create Payment**: `POST /api/v1/payments` (requires JWT)
- **Get Payment**: `GET /api/v1/payment_intents/{intent_id}` (requires JWT)
- **Stripe Webhook**: `POST /webhooks/stripe`

### Auth Service (API Key Protected)
- **Base URL**: http://localhost:8081
- **Health**: `GET /health` (no auth required)
- **Login**: `POST /api/v1/auth/login` (requires X-API-Key header)
- **Register**: `POST /api/v1/auth/register` (requires X-API-Key header)

### Core Service
- **Base URL**: http://localhost:8082
- **Health**: `GET /api/health`

### HAProxy Stats
- **Dashboard**: http://localhost:8404/stats

## Authentication & Security

### 1. API Key (for Backend Services)
```bash
# Auth Service requires API key for all endpoints except /health
curl -X POST http://localhost:8081/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key-from-env" \
  -H "X-Real-IP: 192.168.1.100" \
  -d '{"email":"user@example.com","password":"password"}'
```

### 2. JWT Token (for End Users)
```bash
# Login to get JWT token
TOKEN=$(curl -X POST http://localhost:8081/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{"email":"user@example.com","password":"password"}' \
  | jq -r '.token')

# Use token for API requests
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/v1/payments
```

## Rate Limiting

Both services have rate limiting enabled:

- **Auth Service**: 10 requests/minute per IP
- **Gateway**: 10 requests/minute per user

Rate limit headers:
```
X-RateLimit-Limit: 10
X-RateLimit-Remaining: 8
X-RateLimit-Retry-After: 0
```

## Features

- ✅ JWT Authentication
- ✅ API Key Authentication (for backend services)
- ✅ Token Bucket Rate Limiting (10 req/min)
- ✅ Redis Caching (24h TTL for payments)
- ✅ HAProxy Load Balancing (3 gateway instances)
- ✅ Kafka Event Streaming
- ✅ Stripe Payment Integration
- ✅ Docker Multi-stage Builds with Cargo Chef
- ✅ Real IP Detection (X-Real-IP, X-Forwarded-For)

## Documentation

See [docs/README.md](./docs/README.md) for detailed documentation on:
- Architecture overview
- JWT & API Key authentication
- Rate limiting implementation
- Redis caching strategy
- Load balancing setup
- Docker build optimization
- Kafka event streaming

## Troubleshooting

### Rust not found after installation
```bash
# Restart terminal/PowerShell
# Or reload PATH:
$env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
```

### rust-analyzer not working
```bash
# Install component
rustup component add rust-analyzer

# Restart VS Code
```

### Docker build slow
First build takes ~10 minutes (dependencies). Subsequent builds with code changes only take ~30 seconds thanks to Cargo Chef caching.

### Rate limit errors
Check rate limit headers in response. Wait for `X-RateLimit-Retry-After` seconds or use different IP/user.

### API Key unauthorized
Ensure `AUTH_API_KEYS` is set in `.env` file and matches the `X-API-Key` header value.
