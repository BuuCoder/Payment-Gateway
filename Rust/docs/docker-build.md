# Docker Build với Cargo Chef

## 🔍 Cargo Chef là gì?

**Cargo Chef** là công cụ tối ưu hóa Docker build cho Rust projects bằng cách cache dependencies riêng biệt với source code.

**Ví dụ đơn giản:**
- Giống như nấu ăn: chuẩn bị nguyên liệu (dependencies) trước, sau đó mới nấu (compile code)
- Nguyên liệu không đổi → Không cần mua lại
- Chỉ thay đổi cách nấu → Nhanh hơn nhiều

---

## 🎯 Phục vụ vấn đề gì?

### Vấn đề: Build Docker chậm

**Không có Cargo Chef:**
```
Lần 1: Build tất cả (dependencies + code) → 10 phút
Sửa 1 dòng code
Lần 2: Build lại tất cả (dependencies + code) → 10 phút ❌
```

**Vấn đề:**
- Mỗi lần sửa code phải compile lại dependencies (không thay đổi)
- Tốn thời gian, tốn tài nguyên
- Developer chờ lâu, productivity giảm

### Giải pháp: Cargo Chef + Layer Caching

**Có Cargo Chef:**
```
Lần 1: Build dependencies (cache) → 10 phút
       Build code → 30 giây
       Tổng: ~10 phút

Sửa 1 dòng code (dependencies không đổi)

Lần 2: Dùng cached dependencies → 0 giây
       Build code → 30 giây
       Tổng: ~30 giây ✅
```

**Lợi ích:**
- ✅ Build nhanh gấp 20 lần khi chỉ sửa code
- ✅ Tiết kiệm tài nguyên CI/CD
- ✅ Developer productivity tăng cao

---

## 🏗️ Vai trò trong Source Code

### 1. **Multi-stage Docker Build**
```dockerfile
# Stage 1: Planner - Phân tích dependencies
FROM rust:1.75 as planner
WORKDIR /app
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Cacher - Build và cache dependencies
FROM rust:1.75 as cacher
WORKDIR /app
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Stage 3: Builder - Build source code
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
COPY --from=cacher /app/target target
RUN cargo build --release

# Stage 4: Runtime - Image cuối cùng (nhỏ gọn)
FROM debian:bookworm-slim
COPY --from=builder /app/target/release/app /usr/local/bin/
CMD ["app"]
```

### 2. **Layer Caching Strategy**
- **Layer 1**: Cargo Chef installation (hiếm khi thay đổi)
- **Layer 2**: Dependencies build (chỉ thay đổi khi sửa Cargo.toml)
- **Layer 3**: Source code build (thay đổi thường xuyên)

---

## 📖 Cách sử dụng

### 1. Build tất cả services

```bash
cd Rust
docker compose -f infra/compose.yml build
```

**Output:**
```
[+] Building 600s (45/45) FINISHED
 => [gateway planner] ...
 => [gateway cacher] ...
 => [gateway builder] ...
 => [auth-service planner] ...
```

### 2. Build một service cụ thể

```bash
# Build chỉ Gateway
docker compose -f infra/compose.yml build gateway

# Build chỉ Auth Service
docker compose -f infra/compose.yml build auth-service
```

### 3. Start services sau khi build

```bash
# Start tất cả services
docker compose -f infra/compose.yml up -d

# Start service cụ thể
docker compose -f infra/compose.yml up -d gateway
```

### 4. Rebuild và restart (một lệnh)

```bash
# Rebuild và restart Gateway
docker compose -f infra/compose.yml up -d --build gateway
```

---

## 🔄 Workflow khi sửa code

### Scenario 1: Chỉ sửa code (không đổi dependencies)

**Thời gian: ~30 giây**

```bash
# 1. Sửa code trong services/gateway/src/main.rs
vim services/gateway/src/main.rs

# 2. Build (sử dụng cached dependencies)
docker compose -f infra/compose.yml build gateway
# ✅ Cached: [gateway cacher] (dependencies)
# 🔨 Building: [gateway builder] (code only)

# 3. Restart service
docker compose -f infra/compose.yml up -d gateway
```

### Scenario 2: Thêm/sửa dependencies

**Thời gian: ~5-7 phút**

```bash
# 1. Sửa Cargo.toml (thêm dependency mới)
vim services/gateway/Cargo.toml

# 2. Build (rebuild dependencies layer)
docker compose -f infra/compose.yml build gateway
# 🔨 Building: [gateway cacher] (dependencies)
# 🔨 Building: [gateway builder] (code)

# 3. Restart service
docker compose -f infra/compose.yml up -d gateway
```

---

## 📊 So sánh Build Time

| Tình huống | Không có Cargo Chef | Có Cargo Chef | Tiết kiệm |
|------------|---------------------|---------------|-----------|
| **Build lần đầu** | ~10 phút | ~10 phút | 0% |
| **Sửa code** | ~10 phút | ~30 giây | **95%** ✅ |
| **Thêm dependency** | ~10 phút | ~5-7 phút | ~40% |
| **Rebuild toàn bộ** | ~10 phút | ~10 phút | 0% |

---

## ✅ Khi nào nên dùng?

| Tình huống | Nên dùng? | Lý do |
|------------|-----------|-------|
| Production deployment | ✅ Có | Consistent environment, reproducible builds |
| CI/CD pipelines | ✅ Có | Tiết kiệm thời gian build, giảm chi phí |
| Team collaboration | ✅ Có | Đảm bảo mọi người cùng environment |
| Development (sửa code thường xuyên) | ✅ Có | Build nhanh, productivity cao |
| Quick local testing | ❌ Không | Dùng `cargo run` nhanh hơn |
| Prototype/POC | ❌ Không | Overhead không cần thiết |

---

## 🛠️ Troubleshooting

### Build chậm bất thường

```bash
# Xóa cache và rebuild từ đầu
docker compose -f infra/compose.yml build --no-cache gateway

# Xóa tất cả images cũ
docker system prune -a
```

### Dependency conflict

```bash
# Update Cargo.lock
cd services/gateway
cargo update

# Rebuild
cd ../..
docker compose -f infra/compose.yml build gateway
```

### Kiểm tra logs

```bash
# Xem build logs
docker compose -f infra/compose.yml build gateway --progress=plain

# Xem runtime logs
docker compose -f infra/compose.yml logs -f gateway
```

---

## 💡 Tóm tắt

**Docker Build với Cargo Chef** tối ưu build time cho Rust projects:
- **Mục đích**: Cache dependencies riêng, giảm thời gian build
- **Cách hoạt động**: Multi-stage build với layer caching
- **Vai trò**: Tăng productivity, tiết kiệm tài nguyên CI/CD
- **Kết quả**: Build nhanh gấp 20 lần khi chỉ sửa code (10 phút → 30 giây)
