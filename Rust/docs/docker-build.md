# Docker Build với Cargo Chef

## Tác dụng
- Tối ưu build time với layer caching
- Build lần đầu: ~10 phút
- Build lần sau (chỉ sửa code): ~30 giây

## Cách dùng

### Build tất cả services
```bash
cd Rust
docker compose -f infra/compose.yml build
```

### Build một service
```bash
docker compose -f infra/compose.yml build gateway
```

### Start services
```bash
docker compose -f infra/compose.yml up -d
```

### Rebuild và restart
```bash
docker compose -f infra/compose.yml up -d --build gateway
```

## Workflow khi sửa code

### Chỉ sửa code (không đổi dependencies)
```bash
# 1. Sửa code trong services/ hoặc crates/
# 2. Build (~30 giây)
docker compose -f infra/compose.yml build gateway
# 3. Restart
docker compose -f infra/compose.yml up -d gateway
```

### Thêm/sửa dependencies
```bash
# 1. Sửa Cargo.toml
# 2. Build (~5-7 phút)
docker compose -f infra/compose.yml build gateway
# 3. Restart
docker compose -f infra/compose.yml up -d gateway
```

## Khi nào dùng
- ✅ Production deployment
- ✅ CI/CD pipelines
- ✅ Team collaboration (consistent environment)
- ❌ Quick local testing (dùng `cargo run`)
