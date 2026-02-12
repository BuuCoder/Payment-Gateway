# Redis Cache

## Tác dụng
- Tăng tốc độ truy vấn (5-10ms thay vì 200-500ms)
- Giảm tải database 70-90%
- Lưu trữ payment data với TTL 24 giờ

## Cách dùng

### Get/Set Cache
```rust
// Get
let payment = redis_cache.get::<Payment>(&key)?;

// Set với TTL
redis_cache.set(&key, &payment, 86400)?;

// Delete
redis_cache.delete(&key)?;
```

### Clear Cache
```bash
# Clear tất cả payment cache
docker exec infra-redis-1 redis-cli --scan --pattern "payment:*" | xargs docker exec -i infra-redis-1 redis-cli DEL

# Clear specific key
docker exec infra-redis-1 redis-cli DEL "payment:pi_xxx"
```

## Khi nào dùng
- ✅ Data được đọc nhiều lần (payment info)
- ✅ Data ít thay đổi hoặc có TTL phù hợp
- ✅ Cần giảm tải database
- ❌ Data thay đổi liên tục
- ❌ Data cần consistency tuyệt đối
