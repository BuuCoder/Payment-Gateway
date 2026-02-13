# Redis Cache

## 🔍 Redis Cache là gì?

**Redis Cache** là một hệ thống lưu trữ dữ liệu tạm thời (cache) trong bộ nhớ RAM, giúp truy xuất dữ liệu cực nhanh.

**Ví dụ đơn giản:**
- Giống như bạn ghi nhớ số điện thoại người thân → không cần mở danh bạ mỗi lần gọi
- Database = danh bạ (chậm nhưng đầy đủ)
- Redis Cache = trí nhớ (nhanh nhưng tạm thời)

---

## 🎯 Phục vụ vấn đề gì?

### Vấn đề trước khi có Redis Cache:
```
User request → Database query → Chờ 200-500ms → Response
```
- ❌ **Chậm**: Mỗi lần cần dữ liệu phải query database
- ❌ **Tốn tài nguyên**: Database bị quá tải khi nhiều request
- ❌ **Trải nghiệm kém**: User phải chờ lâu

### Giải pháp với Redis Cache:
```
User request → Redis Cache (5-10ms) → Response nhanh
              ↓ (nếu không có trong cache)
              Database → Lưu vào cache → Response
```
- ✅ **Nhanh gấp 20-50 lần**: 5-10ms thay vì 200-500ms
- ✅ **Giảm tải database**: 70-90% request không cần đụng database
- ✅ **Trải nghiệm tốt**: User nhận response gần như tức thì

---

## 🏗️ Vai trò trong Source Code

### 1. **Caching Payment Data**
Khi user kiểm tra thông tin thanh toán:
- Lần đầu: Query từ database → Lưu vào Redis với TTL 24 giờ
- Lần sau: Lấy trực tiếp từ Redis (nhanh hơn nhiều)

### 2. **Giảm tải Database**
- Database chỉ xử lý request khi cache hết hạn hoặc chưa có
- Phần lớn request được Redis xử lý

### 3. **TTL (Time To Live)**
- Dữ liệu tự động xóa sau 24 giờ
- Đảm bảo dữ liệu không quá cũ

---

## 📖 Cách sử dụng trong Code

### 1. Lấy dữ liệu từ Cache
```rust
// Lấy payment từ Redis
let payment = redis_cache.get::<Payment>(&key)?;
```

### 2. Lưu dữ liệu vào Cache
```rust
// Lưu payment vào Redis với TTL 24 giờ (86400 giây)
redis_cache.set(&key, &payment, 86400)?;
```

### 3. Xóa dữ liệu khỏi Cache
```rust
// Xóa payment khỏi Redis
redis_cache.delete(&key)?;
```

---

## 🛠️ Quản lý Cache (DevOps)

### Xóa toàn bộ payment cache
```bash
docker exec infra-redis-1 redis-cli --scan --pattern "payment:*" | \
  xargs docker exec -i infra-redis-1 redis-cli DEL
```

### Xóa một key cụ thể
```bash
docker exec infra-redis-1 redis-cli DEL "payment:pi_xxx"
```

---

## ✅ Khi nào nên dùng Redis Cache?

| Tình huống | Nên dùng? | Lý do |
|------------|-----------|-------|
| Dữ liệu được đọc nhiều lần (payment info) | ✅ Có | Tối ưu tốc độ, giảm tải DB |
| Dữ liệu ít thay đổi hoặc có TTL phù hợp | ✅ Có | Cache vẫn chính xác |
| Cần giảm tải database | ✅ Có | Redis xử lý phần lớn request |
| Dữ liệu thay đổi liên tục | ❌ Không | Cache sẽ luôn sai lệch |
| Dữ liệu cần tính nhất quán tuyệt đối | ❌ Không | Nên query trực tiếp DB |

---

## 📊 Hiệu suất thực tế

| Metric | Trước khi có Redis | Sau khi có Redis |
|--------|-------------------|------------------|
| Thời gian response | 200-500ms | 5-10ms |
| Tải database | 100% | 10-30% |
| Số lượng query DB | Mọi request | Chỉ khi cache miss |

---

## 💡 Tóm tắt

**Redis Cache** là lớp lưu trữ tạm thời giữa application và database:
- **Mục đích**: Tăng tốc độ, giảm tải database
- **Cách hoạt động**: Lưu dữ liệu thường xuyên truy cập vào RAM
- **Vai trò**: Caching layer cho payment data với TTL 24 giờ
- **Kết quả**: Response nhanh hơn 20-50 lần, giảm 70-90% tải database
