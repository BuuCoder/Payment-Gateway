# API Key Authentication

## Tác dụng
- Bảo vệ Auth Service khỏi truy cập trái phép
- Chỉ cho phép backend services có API key hợp lệ
- Hỗ trợ X-Real-IP header để rate limit theo IP thực của end-user

## Use Case: Backend Proxy Pattern

### Vấn đề
```
100 users → NodeJS Backend → Rust Auth Service
              (1 IP)           (thấy 1 IP)
```

Nếu rate limit theo connection IP, 100 users khác nhau sẽ bị coi là 1 user!

### Giải pháp
```
100 users → NodeJS Backend → Rust Auth Service
            (forward real IP)  (rate limit theo real IP)
```

NodeJS backend:
1. Có API key hợp lệ
2. Truyền IP thực của end-user qua header `X-Real-IP`
3. Rust Auth Service rate limit theo IP thực

## Cấu hình

### 1. Set API Keys (Environment Variable)
```bash
# .env file
AUTH_API_KEYS=nodejs-backend-key-abc123,mobile-app-key-xyz789
```

### 2. NodeJS Backend Example
```javascript
const axios = require('axios');

// Login endpoint của NodeJS
app.post('/api/login', async (req, res) => {
  const { email, password } = req.body;
  const clientIP = req.ip || req.connection.remoteAddress;
  
  try {
    // Forward request to Rust Auth Service
    const response = await axios.post('http://rust-auth:8081/api/v1/auth/login', {
      email,
      password
    }, {
      headers: {
        'X-API-Key': 'nodejs-backend-key-abc123',  // API key
        'X-Real-IP': clientIP,                      // Real client IP
        'Content-Type': 'application/json'
      }
    });
    
    res.json(response.data);
  } catch (error) {
    if (error.response?.status === 429) {
      // Rate limited
      res.status(429).json({
        error: 'Too many login attempts',
        retry_after: error.response.headers['x-ratelimit-retry-after']
      });
    } else {
      res.status(error.response?.status || 500).json({
        error: error.response?.data?.error || 'Login failed'
      });
    }
  }
});
```

### 3. Test với curl
```bash
# Without API key → 401 Unauthorized
curl -X POST http://localhost:8081/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"password"}'

# With API key → Success
curl -X POST http://localhost:8081/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -H "X-API-Key: nodejs-backend-key-abc123" \
  -H "X-Real-IP: 192.168.1.100" \
  -d '{"email":"user@example.com","password":"password"}'
```

## IP Detection Priority

1. **X-Real-IP** header (recommended for trusted backends)
2. **X-Forwarded-For** header (first IP in chain)
3. **Connection IP** (fallback)

### Example Headers
```http
X-Real-IP: 192.168.1.100
X-Forwarded-For: 192.168.1.100, 10.0.0.1, 172.16.0.1
```

Rate limiter sẽ dùng `192.168.1.100` (từ X-Real-IP)

## Security Best Practices

### 1. Generate Strong API Keys
```bash
# Linux/Mac
openssl rand -hex 32

# Output: 64-character hex string
# Example: a1b2c3d4e5f6...
```

### 2. Rotate Keys Regularly
```bash
# Add new key first
AUTH_API_KEYS=old-key,new-key

# Update all backends to use new key
# Then remove old key
AUTH_API_KEYS=new-key
```

### 3. Different Keys per Backend
```bash
AUTH_API_KEYS=nodejs-key-abc,mobile-key-xyz,admin-panel-key-123
```

Dễ dàng revoke access của 1 backend mà không ảnh hưởng các backend khác.

### 4. Never Commit Keys to Git
```bash
# .env (gitignored)
AUTH_API_KEYS=real-production-key

# .env.example (committed)
AUTH_API_KEYS=your-secure-api-key-here
```

## Rate Limiting với API Key

### Scenario 1: 100 Users qua NodeJS Backend
```
User A (IP: 1.1.1.1) → NodeJS → Auth (rate limit: 1.1.1.1)
User B (IP: 2.2.2.2) → NodeJS → Auth (rate limit: 2.2.2.2)
User C (IP: 3.3.3.3) → NodeJS → Auth (rate limit: 3.3.3.3)
```

Mỗi user có rate limit riêng (10 attempts burst, 60/min sustained)

### Scenario 2: Không có X-Real-IP
```
100 Users → NodeJS (IP: 10.0.0.1) → Auth (rate limit: 10.0.0.1)
```

Tất cả 100 users share chung 1 rate limit bucket → Không tốt!

## Health Check Endpoint

Health check endpoint `/health` không cần API key:
```bash
curl http://localhost:8081/health
# Response: {"status":"ok","service":"auth-service"}
```

## Error Responses

### Missing API Key
```json
{
  "error": "Unauthorized",
  "message": "Valid API key required"
}
```

### Invalid API Key
```json
{
  "error": "Unauthorized",
  "message": "Valid API key required"
}
```

### Rate Limited (with valid API key)
```json
{
  "error": "Too many attempts. Please try again later.",
  "retry_after_seconds": 5,
  "limit": 10,
  "remaining": 0
}
```

## Khi nào dùng

### ✅ Nên dùng
- Backend services làm proxy cho end-users
- Microservices internal communication
- Mobile apps gọi trực tiếp (1 key per app)
- Admin panels với elevated privileges

### ❌ Không nên dùng
- Public APIs cho end-users trực tiếp (dùng JWT)
- Browser-based SPAs (key sẽ bị expose)
- Open APIs không cần authentication

## Production Checklist

- [ ] Generate strong API keys (32+ characters)
- [ ] Set AUTH_API_KEYS environment variable
- [ ] Configure backend to send X-Real-IP header
- [ ] Test rate limiting với multiple IPs
- [ ] Monitor unauthorized access attempts
- [ ] Set up key rotation schedule
- [ ] Document which backend uses which key
- [ ] Never log API keys in plaintext
