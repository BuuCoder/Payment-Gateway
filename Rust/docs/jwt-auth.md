# JWT Authentication

## 🔍 JWT Authentication là gì?

**JWT (JSON Web Token)** là phương thức xác thực stateless, token chứa thông tin user và được ký bởi server để đảm bảo tính toàn vẹn.

**Ví dụ đơn giản:**
- Giống như vé xem phim: có thông tin (tên phim, ghế, giờ chiếu) và chữ ký của rạp
- Nhân viên kiểm tra chữ ký → Biết vé thật hay giả
- Không cần tra cứu database → Nhanh!

---

## 🎯 Phục vụ vấn đề gì?

### Vấn đề với Session-based Authentication

```
❌ Session-based:
┌─────────┐                    ┌─────────┐
│ Client  │ ──── Request ────► │ Server  │
│         │                    │         │
│         │                    │ Check   │
│         │                    │ Session │
│         │                    │ in DB   │
│         │ ◄─── Response ──── │         │
└─────────┘                    └─────────┘
                                    │
                                    ▼
                            ┌──────────────┐
                            │ Session DB   │
                            │ (Redis/Mem)  │
                            └──────────────┘

Vấn đề:
- Phải lưu session trong database/Redis
- Mỗi request phải query session → Chậm
- Microservices phải share session store → Phức tạp
- Scale khó: Cần sync session giữa servers
```

### Giải pháp với JWT

```
✅ JWT (Stateless):
┌─────────┐                    ┌─────────┐
│ Client  │ ──── Request ────► │ Server  │
│         │   + JWT Token      │         │
│         │                    │ Verify  │
│         │                    │ Token   │
│         │                    │ (No DB) │
│         │ ◄─── Response ──── │         │
└─────────┘                    └─────────┘

Lợi ích:
- Không cần lưu session → Stateless
- Verify token nhanh (chỉ kiểm tra chữ ký)
- Microservices độc lập (không cần shared store)
- Scale dễ dàng: Mỗi server tự verify
```

---

## 🏗️ Vai trò trong Source Code

### 1. **Login → Tạo JWT Token**
```rust
// Auth Service: User login
POST /api/v1/auth/login
{
  "email": "user@example.com",
  "password": "password"
}

// Response: JWT token
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": { "id": 123, "email": "user@example.com" }
}
```

### 2. **Middleware tự động verify JWT**
```rust
// Gateway: Mọi request đều qua middleware
async fn jwt_middleware(
    req: Request,
    next: Next,
) -> Result<Response> {
    // 1. Extract token từ header
    let token = extract_token(&req)?;
    
    // 2. Verify token (kiểm tra chữ ký, expiration)
    let claims = verify_jwt(&token)?;
    
    // 3. Inject claims vào request
    req.extensions_mut().insert(claims);
    
    // 4. Tiếp tục xử lý request
    next.run(req).await
}
```

### 3. **Handler sử dụng user info từ JWT**
```rust
// Handler tự động nhận Claims (đã verify)
async fn get_payments(
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<Payment>>> {
    let user_id = claims.user_id;  // Đã verify, an toàn
    
    // Query payments của user
    let payments = db.get_payments(user_id).await?;
    Ok(Json(payments))
}
```

---

## 📖 Cách sử dụng

### 1. Login để lấy token

```bash
# Login request
curl -X POST http://localhost:8081/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "password"
  }'

# Response
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VyX2lkIjoxMjMsImVtYWlsIjoidXNlckBleGFtcGxlLmNvbSIsImV4cCI6MTcwOTQ3MjAwMH0.abc123...",
  "user": {
    "id": 123,
    "email": "user@example.com"
  }
}
```

### 2. Sử dụng token trong requests

```bash
# Lưu token vào biến
TOKEN="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# Gọi API với token
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/v1/payments

curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/v1/users/me
```

### 3. Trong code (middleware tự động)

```rust
// Middleware tự động extract và verify
async fn handler(
    Extension(claims): Extension<Claims>,
) -> Result<Json<Response>> {
    // claims.user_id đã được verify
    // claims.email đã được verify
    // claims.exp đã được kiểm tra (chưa hết hạn)
    
    let user_id = claims.user_id;
    // Xử lý logic...
}
```

---

## 🔐 Token Structure

### JWT Format
```
eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VyX2lkIjoxMjMsImVtYWlsIjoidXNlckBleGFtcGxlLmNvbSIsImV4cCI6MTcwOTQ3MjAwMH0.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c

│────────── Header ──────────│──────────── Payload ────────────│────── Signature ──────│
```

### Header (Base64 encoded)
```json
{
  "alg": "HS256",      // Algorithm: HMAC SHA256
  "typ": "JWT"         // Type: JWT
}
```

### Payload (Base64 encoded) - Claims
```json
{
  "user_id": 123,
  "email": "user@example.com",
  "exp": 1709472000    // Expiration timestamp
}
```

### Signature
```
HMACSHA256(
  base64UrlEncode(header) + "." + base64UrlEncode(payload),
  secret_key
)
```

**Lưu ý:** Token có thể decode để đọc (Base64), nhưng không thể sửa đổi (có signature)

---

## ⏱️ Token Expiration

```rust
// Token hết hạn sau 24 giờ
let expiration = Utc::now() + Duration::hours(24);

let claims = Claims {
    user_id: 123,
    email: "user@example.com".to_string(),
    exp: expiration.timestamp(),
};
```

**Khi token hết hạn:**
```bash
# Request với token hết hạn
curl -H "Authorization: Bearer $EXPIRED_TOKEN" \
  http://localhost:8080/api/v1/payments

# Response: 401 Unauthorized
{
  "error": "Token expired"
}
```

**Giải pháp:** User phải login lại để lấy token mới

---

## 🔄 Workflow hoàn chỉnh

```
1. User Login
   ↓
   POST /api/v1/auth/login
   ↓
   Server verify password
   ↓
   Server tạo JWT token (chứa user_id, email, exp)
   ↓
   Response: { token: "eyJ..." }

2. User gọi API
   ↓
   GET /api/v1/payments
   Header: Authorization: Bearer eyJ...
   ↓
   Middleware verify JWT
   ↓
   Extract claims (user_id, email)
   ↓
   Handler xử lý với user_id
   ↓
   Response: [...payments...]

3. Token hết hạn
   ↓
   User login lại
   ↓
   Lấy token mới
```

---

## ✅ Khi nào nên dùng?

| Tình huống | Nên dùng? | Lý do |
|------------|-----------|-------|
| Microservices | ✅ Có | Không cần shared session store |
| Mobile/SPA apps | ✅ Có | Dễ lưu token, gửi kèm mỗi request |
| API authentication | ✅ Có | Stateless, scale dễ dàng |
| RESTful APIs | ✅ Có | Phù hợp với REST principles |
| Cần revoke token ngay lập tức | ❌ Không | JWT không thể revoke (dùng session) |
| Long-lived sessions | ❌ Không | Dùng refresh token pattern |
| Highly sensitive operations | ❌ Không | Cân nhắc thêm 2FA, session |

---

## 🛡️ Security Best Practices

### 1. Secret Key Management
```bash
# .env - NEVER commit to git
JWT_SECRET=your-super-secret-key-at-least-32-characters-long

# Generate strong secret
openssl rand -hex 32
```

### 2. Token Expiration
```rust
// Short-lived tokens (1-24 hours)
let expiration = Utc::now() + Duration::hours(24);

// Refresh token pattern cho long sessions
// Access token: 15 phút
// Refresh token: 7 ngày
```

### 3. HTTPS Only
```
❌ HTTP: Token bị intercept → Mất bảo mật
✅ HTTPS: Token được mã hóa → An toàn
```

### 4. Validate Claims
```rust
// Luôn kiểm tra expiration
if claims.exp < Utc::now().timestamp() {
    return Err("Token expired");
}

// Validate user_id exists
let user = db.get_user(claims.user_id).await?;
```

---

## 💡 Tóm tắt

**JWT Authentication** là phương thức xác thực stateless:
- **Mục đích**: Xác thực user không cần session store
- **Cách hoạt động**: Token chứa user info + signature, server verify signature
- **Vai trò**: Middleware tự động verify, handler nhận claims đã verify
- **Kết quả**: Stateless, scale dễ, phù hợp microservices và mobile/SPA apps
