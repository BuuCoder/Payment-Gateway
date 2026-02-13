# API Key Authentication

## 🔍 API Key Authentication là gì?

**API Key Authentication** là phương thức xác thực dựa trên một chuỗi ký tự bí mật (API key) được gửi kèm trong mỗi request để xác minh danh tính của client.

**Ví dụ đơn giản:**
- Giống như mật khẩu của ứng dụng (không phải của user)
- Backend service cần "chìa khóa" để truy cập Auth Service
- Mỗi backend có một chìa khóa riêng

---

## 🎯 Phục vụ vấn đề gì?

### Vấn đề 1: Bảo mật Auth Service
```
❌ Không có API Key:
Bất kỳ ai cũng có thể gọi Auth Service → Nguy hiểm!

✅ Có API Key:
Chỉ backend services được ủy quyền mới gọi được → An toàn!
```

### Vấn đề 2: Backend Proxy Pattern

**Tình huống thực tế:**
```
100 users → NodeJS Backend (IP: 10.0.0.1) → Rust Auth Service
```

**Vấn đề:** Auth Service chỉ thấy 1 IP (10.0.0.1) của NodeJS Backend
- Rate limiting sẽ giới hạn tất cả 100 users như 1 user
- User A bị rate limit vì User B spam requests
- Không công bằng!

**Giải pháp với X-Real-IP:**
```
User A (1.1.1.1) ──┐
User B (2.2.2.2) ──┼──► NodeJS Backend ──► Auth Service
User C (3.3.3.3) ──┘     (forward real IP)  (rate limit từng IP)
```

NodeJS Backend gửi kèm:
1. **X-API-Key**: Chứng minh backend hợp lệ
2. **X-Real-IP**: IP thực của end-user
3. Auth Service rate limit theo IP thực → Mỗi user độc lập

---

## 🏗️ Vai trò trong Source Code

### 1. **Middleware Bảo vệ Auth Service**
- Kiểm tra mọi request đến Auth Service
- Chỉ cho phép requests có API key hợp lệ
- Từ chối tất cả requests không có hoặc sai API key

### 2. **Hỗ trợ Backend Proxy Pattern**
- Nhận IP thực từ header `X-Real-IP`
- Rate limiting chính xác theo từng end-user
- Ngăn chặn brute force attacks hiệu quả

### 3. **Quản lý Access Control**
- Mỗi backend có key riêng
- Dễ dàng revoke access khi cần
- Audit log theo từng backend

---

## 📖 Cách sử dụng

### 1. Cấu hình API Keys

**Tạo API key mạnh:**
```bash
# Linux/Mac
openssl rand -hex 32
# Output: a1b2c3d4e5f6789...abc123 (64 ký tự)

# Windows PowerShell
[Convert]::ToBase64String((1..32 | ForEach-Object { Get-Random -Maximum 256 }))
```

**Thêm vào environment variable:**
```bash
# .env file
AUTH_API_KEYS=nodejs-backend-key-abc123,mobile-app-key-xyz789,admin-key-def456
```

### 2. NodeJS Backend Example

```javascript
const axios = require('axios');

app.post('/api/login', async (req, res) => {
  const { email, password } = req.body;
  
  // Lấy IP thực của client
  const clientIP = req.ip || req.connection.remoteAddress;
  
  try {
    // Gọi Rust Auth Service
    const response = await axios.post(
      'http://rust-auth:8081/api/v1/auth/login',
      { email, password },
      {
        headers: {
          'X-API-Key': 'nodejs-backend-key-abc123',  // API key
          'X-Real-IP': clientIP,                      // IP thực
          'Content-Type': 'application/json'
        }
      }
    );
    
    res.json(response.data);
  } catch (error) {
    // Xử lý lỗi
    res.status(error.response?.status || 500).json({
      error: error.response?.data?.error || 'Login failed'
    });
  }
});
```

### 3. Test với curl

**Không có API key → 401 Unauthorized:**
```bash
curl -X POST http://localhost:8081/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"password"}'
```

**Có API key → Success:**
```bash
curl -X POST http://localhost:8081/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -H "X-API-Key: nodejs-backend-key-abc123" \
  -H "X-Real-IP: 192.168.1.100" \
  -d '{"email":"user@example.com","password":"password"}'
```

---

## 🔐 IP Detection Priority

Auth Service tìm IP theo thứ tự ưu tiên:

| Thứ tự | Header | Mô tả | Khi nào dùng |
|--------|--------|-------|--------------|
| 1 | `X-Real-IP` | IP thực từ trusted backend | ✅ Recommended |
| 2 | `X-Forwarded-For` | IP đầu tiên trong chain | Qua nhiều proxy |
| 3 | Connection IP | IP của kết nối trực tiếp | Fallback |

**Ví dụ:**
```http
X-Real-IP: 192.168.1.100
X-Forwarded-For: 192.168.1.100, 10.0.0.1, 172.16.0.1
```
→ Rate limiter sẽ dùng `192.168.1.100` (từ X-Real-IP)

---

## 🛡️ Security Best Practices

### 1. Generate Strong Keys
```bash
# Tối thiểu 32 ký tự
openssl rand -hex 32

# ✅ Tốt: a1b2c3d4e5f6789...abc123 (64 chars)
# ❌ Tệ: dev-key-123 (quá ngắn, dễ đoán)
```

### 2. Rotate Keys Regularly
```bash
# Bước 1: Thêm key mới (giữ key cũ)
AUTH_API_KEYS=old-key,new-key

# Bước 2: Update tất cả backends dùng new-key
# Bước 3: Xóa old-key
AUTH_API_KEYS=new-key
```

### 3. Different Keys per Backend
```bash
AUTH_API_KEYS=nodejs-key-abc,mobile-key-xyz,admin-key-def
```
**Lợi ích:**
- Revoke access của 1 backend mà không ảnh hưởng backend khác
- Dễ dàng audit: biết backend nào gọi API
- Tăng security: 1 key bị lộ không ảnh hưởng toàn bộ

### 4. Never Commit Keys to Git
```bash
# .env (gitignored) - Chứa key thật
AUTH_API_KEYS=real-production-key-a1b2c3d4e5f6

# .env.example (committed) - Chỉ là template
AUTH_API_KEYS=your-secure-api-key-here
```

---

## 📊 Rate Limiting Scenarios

### Scenario 1: Có X-Real-IP (✅ Đúng)
```
User A (1.1.1.1) → NodeJS → Auth Service
                            Rate limit: 1.1.1.1 (10 req/min)

User B (2.2.2.2) → NodeJS → Auth Service
                            Rate limit: 2.2.2.2 (10 req/min)
```
→ Mỗi user có quota riêng, công bằng!

### Scenario 2: Không có X-Real-IP (❌ Sai)
```
100 Users → NodeJS (10.0.0.1) → Auth Service
                                 Rate limit: 10.0.0.1 (10 req/min)
```
→ Tất cả 100 users share chung 10 req/min → Không tốt!

---

## 🚫 Error Responses

| Lỗi | HTTP Status | Response | Nguyên nhân |
|-----|-------------|----------|-------------|
| Missing API Key | 401 | `{"error":"Unauthorized","message":"Valid API key required"}` | Không gửi header X-API-Key |
| Invalid API Key | 401 | `{"error":"Unauthorized","message":"Valid API key required"}` | API key sai hoặc không tồn tại |
| Rate Limited | 429 | `{"error":"Too many attempts...","retry_after_seconds":5}` | Vượt quá giới hạn request |

---

## ✅ Khi nào nên dùng?

| Tình huống | Nên dùng? | Lý do |
|------------|-----------|-------|
| Backend services làm proxy cho end-users | ✅ Có | Bảo vệ Auth Service, rate limit chính xác |
| Microservices internal communication | ✅ Có | Xác thực giữa các services |
| Mobile apps gọi trực tiếp | ✅ Có | 1 key per app, dễ quản lý |
| Admin panels | ✅ Có | Elevated privileges, audit log |
| Public APIs cho end-users | ❌ Không | Dùng JWT thay vì API key |
| Browser-based SPAs | ❌ Không | API key sẽ bị expose trong code |
| Open APIs không cần auth | ❌ Không | Không cần bảo mật |

---

## 📋 Production Checklist

- [ ] Generate strong API keys (32+ characters)
- [ ] Set `AUTH_API_KEYS` environment variable
- [ ] Configure backend to send `X-Real-IP` header
- [ ] Test rate limiting với multiple IPs
- [ ] Monitor unauthorized access attempts (401 errors)
- [ ] Set up key rotation schedule (mỗi 3-6 tháng)
- [ ] Document which backend uses which key
- [ ] Never log API keys in plaintext
- [ ] Add `.env` to `.gitignore`
- [ ] Use different keys for dev/staging/production

---

## 💡 Tóm tắt

**API Key Authentication** bảo vệ Auth Service khỏi truy cập trái phép:
- **Mục đích**: Chỉ cho phép backend services được ủy quyền
- **Cách hoạt động**: Kiểm tra header `X-API-Key` trong mỗi request
- **Vai trò**: Middleware bảo vệ + hỗ trợ rate limiting chính xác
- **Kết quả**: Bảo mật cao, rate limiting công bằng cho từng user
