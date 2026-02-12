# Rate Limiting (Token Bucket)

## Tác dụng
- Kiểm soát số lượng requests per user/IP
- Cho phép burst traffic có kiểm soát
- Bảo vệ API khỏi abuse/DDoS/brute force

## Services có Rate Limiting

### Gateway (port 8080)
```
Capacity: 10 tokens
Refill: 0.166 tokens/second (10 req/min)
Identifier: JWT user_id hoặc IP address
```

### Auth Service (port 8081)
```
Capacity: 10 tokens
Refill: 0.166 tokens/second (10 req/min)
Identifier: IP address (X-Real-IP > X-Forwarded-For > Connection IP)
Endpoints: /login, /register
```

## Cách hoạt động
```
Bucket: 10 tokens (capacity)
Refill: 10 tokens/60 seconds = 0.166 tokens/second
Cost: 1 token/request

→ Cho phép 10 requests burst (ngay lập tức)
→ Sau đó 10 requests/minute sustained
→ Sau 60 giây, bucket đầy lại (10 tokens)
```

## Cấu hình
```rust
// Auth Service & Gateway (strict - 10 requests/minute)
RateLimiter::new(redis_cache, 10.0, 10.0 / 60.0)

// Nếu cần generous hơn (60 requests/minute)
RateLimiter::new(redis_cache, 10.0, 1.0)

// Internal APIs (600 requests/minute)
RateLimiter::new(redis_cache, 100.0, 10.0)
```

## Test

### Test Auth Service Rate Limiting
```bash
# Test login rate limit (10 requests per minute)
for i in {1..10}; do
  curl -X POST http://localhost:8081/api/v1/auth/login \
    -H "Content-Type: application/json" \
    -H "X-API-Key: iouWN3RyYqinQVLodUloltG2aEzzCIE" \
    -H "X-Real-IP: 192.168.1.100" \
    -d '{"email":"test@example.com","password":"wrong"}'
done

# 11th attempt → 429 Too Many Requests
curl -X POST http://localhost:8081/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -H "X-API-Key: iouWN3RyYqinQVLodUloltG2aEzzCIE" \
  -H "X-Real-IP: 192.168.1.100" \
  -d '{"email":"test@example.com","password":"wrong"}'

# Wait 60 seconds (10 tokens refilled)
sleep 60

# Should work again
curl -X POST http://localhost:8081/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -H "X-API-Key: iouWN3RyYqinQVLodUloltG2aEzzCIE" \
  -H "X-Real-IP: 192.168.1.100" \
  -d '{"email":"user@example.com","password":"password"}'
```

### Test Gateway Rate Limiting
```bash
# Login lấy token
TOKEN=$(curl -X POST http://localhost:8081/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -H "X-API-Key: dev-key-12345" \
  -d '{"email":"user@example.com","password":"password"}' \
  | jq -r '.token')

# Test burst (10 requests OK)
for i in {1..10}; do
  curl -H "Authorization: Bearer $TOKEN" http://localhost:8080/api/v1/payments
done

# Request 11 → 429 Too Many Requests
curl -H "Authorization: Bearer $TOKEN" http://localhost:8080/api/v1/payments

# Wait 60 seconds (10 tokens refilled)
sleep 60

# Should work again (10 more requests)
curl -H "Authorization: Bearer $TOKEN" http://localhost:8080/api/v1/payments
```

## Khi nào dùng
- ✅ Auth endpoints (login/register) - prevent brute force
- ✅ Public APIs cần bảo vệ
- ✅ Endpoints tốn tài nguyên (write, search)
- ✅ Cần kiểm soát per-user/IP usage
- ✅ APIs với strict quota (10 req/min)
- ❌ Internal services đã tin cậy
- ❌ Endpoints đơn giản (health check)

## Security Benefits

### Brute Force Protection
- Login: Giới hạn 10 requests/minute per IP
- Attacker cần ~100 phút để thử 1000 passwords
- Kết hợp với account lockout để bảo vệ tốt hơn

### API Quota Management
- Gateway: Giới hạn 10 requests/minute per user
- Auth Service: Giới hạn 10 requests/minute per IP
- Ngăn chặn abuse và excessive usage
- Fair usage policy enforcement

### DDoS Mitigation
- Ngăn chặn single user/IP overwhelm hệ thống
- Fail-open strategy: continue on Redis error
