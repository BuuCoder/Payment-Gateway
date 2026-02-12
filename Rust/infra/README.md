# Infrastructure - Docker Build Guide

## Tổng quan

Thư mục này chứa các Dockerfile và docker-compose configuration để build và deploy các microservices.

## Cargo Chef - Tối ưu Build Time

Tất cả Dockerfiles đã được tích hợp **cargo-chef** để tối ưu thời gian build:

- **Build lần đầu**: ~10 phút (build dependencies + code)
- **Build lần sau (chỉ sửa code)**: ~30 giây - 1 phút (chỉ rebuild code)
- **Build lần sau (thêm/sửa dependencies)**: ~5-7 phút (rebuild dependencies)

### Cách hoạt động của Cargo Chef

1. Tạo "recipe" từ Cargo.toml (chứa thông tin dependencies)
2. Build dependencies dựa trên recipe → **layer này được cache**
3. Copy source code và build application

Khi bạn chỉ sửa code trong `services/` hoặc `crates/`, Docker sẽ reuse cached dependencies và chỉ rebuild phần code thay đổi.

## Services

- **gateway**: API Gateway (port 8080)
- **auth-service**: Authentication service (port 8081)
- **core-service**: Core business logic (port 8082)
- **worker-service**: Background worker xử lý Kafka events

## Build Commands

### Build tất cả services

```bash
cd Rust
docker compose -f infra/compose.yml build
```

### Build một service cụ thể

```bash
# Build gateway
docker compose -f infra/compose.yml build gateway

# Build auth-service
docker compose -f infra/compose.yml build auth-service

# Build core-service
docker compose -f infra/compose.yml build core-service

# Build worker-service
docker compose -f infra/compose.yml build worker-service
```

### Build với no-cache (force rebuild toàn bộ)

```bash
docker compose -f infra/compose.yml build --no-cache
```

## Run Services

### Start tất cả services

```bash
docker compose -f infra/compose.yml up -d
```

### Start một service cụ thể

```bash
docker compose -f infra/compose.yml up -d gateway
```

### Stop services

```bash
docker compose -f infra/compose.yml down
```

### Restart một service

```bash
docker compose -f infra/compose.yml restart gateway
```

## Logs & Monitoring

### Xem logs tất cả services

```bash
docker compose -f infra/compose.yml logs -f
```

### Xem logs một service cụ thể

```bash
docker compose -f infra/compose.yml logs -f gateway
```

### Kiểm tra trạng thái services

```bash
docker compose -f infra/compose.yml ps
```

## Development Workflow

### Khi sửa code (không thay đổi dependencies)

```bash
# 1. Sửa code trong services/ hoặc crates/
# 2. Build lại service (chỉ mất ~30 giây)
docker compose -f infra/compose.yml build gateway

# 3. Restart service
docker compose -f infra/compose.yml up -d gateway

# 4. Xem logs
docker compose -f infra/compose.yml logs -f gateway
```

### Khi thêm/sửa dependencies trong Cargo.toml

```bash
# 1. Sửa Cargo.toml
# 2. Build lại (mất ~5-7 phút)
docker compose -f infra/compose.yml build gateway

# 3. Restart service
docker compose -f infra/compose.yml up -d gateway
```

## Dockerfile Structure

Mỗi Dockerfile sử dụng multi-stage build với cargo-chef:

```dockerfile
# Stage 1: Prepare recipe
FROM lukemathwalker/cargo-chef:latest-rust-1.88-bookworm AS chef

# Stage 2: Compute recipe (dependencies info)
FROM chef AS planner
COPY Cargo.toml ./
COPY crates ./crates
COPY services ./services
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Build dependencies (CACHED LAYER)
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Stage 4: Build application
COPY Cargo.toml ./
COPY crates ./crates
COPY services ./services
RUN cargo build --release -p <service-name>

# Stage 5: Runtime (minimal image)
FROM debian:bookworm
COPY --from=builder /app/target/release/<service-name> /usr/local/bin/
CMD ["<service-name>"]
```

## Rust Version

Tất cả services sử dụng **Rust 1.88** để đảm bảo:
- Reproducible builds
- Tương thích với dependencies yêu cầu edition 2024
- Consistency across team

## Troubleshooting

### Build bị lỗi "failed to compile cargo-chef"

Đã fix bằng cách sử dụng pre-built cargo-chef image thay vì compile từ source.

### Lỗi "edition2024 is required"

Đã fix bằng cách upgrade Rust lên version 1.88.

### Build vẫn chậm sau lần đầu

Kiểm tra xem có thay đổi Cargo.toml không. Nếu có, dependencies layer sẽ rebuild.

### Service không start

```bash
# Xem logs để debug
docker compose -f infra/compose.yml logs <service-name>

# Kiểm tra container status
docker compose -f infra/compose.yml ps
```

## Tips

- Luôn build từ thư mục `Rust/` (workspace root)
- Sử dụng `-f infra/compose.yml` để chỉ định compose file
- Thêm `-d` flag để chạy services ở background
- Sử dụng `--build` flag khi up để tự động rebuild nếu có thay đổi:
  ```bash
  docker compose -f infra/compose.yml up -d --build
  ```
