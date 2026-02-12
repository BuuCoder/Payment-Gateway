# JWT Authentication

## Tác dụng
- Stateless authentication (không cần session store)
- Token chứa user info (user_id, email, role)
- Tự động verify và extract user từ token

## Cách dùng

### Login để lấy token
```bash
curl -X POST http://localhost:8081/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"password"}' \
  | jq -r '.token'
```

### Sử dụng token
```bash
TOKEN="eyJhbGc..."

curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/payments
```

### Trong code (middleware tự động)
```rust
// Middleware tự động extract Claims
async fn handler(
    Extension(claims): Extension<Claims>,
) -> Result<Json<Response>> {
    let user_id = claims.user_id;  // Đã verify
    // ...
}
```

## Token structure
```json
{
  "user_id": 123,
  "email": "user@example.com",
  "exp": 1234567890
}
```

## Khi nào dùng
- ✅ Microservices (không cần shared session)
- ✅ Mobile/SPA apps
- ✅ API authentication
- ❌ Cần revoke token ngay lập tức (dùng session)
- ❌ Long-lived sessions (dùng refresh token)
