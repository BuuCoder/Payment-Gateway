# Microservices Architecture

## 🔍 Microservices Architecture là gì?

**Microservices Architecture** là kiến trúc chia ứng dụng thành nhiều services nhỏ, độc lập, mỗi service phụ trách một chức năng cụ thể.

**Ví dụ đơn giản:**
- Thay vì 1 ứng dụng lớn làm tất cả → Chia thành nhiều services nhỏ
- Mỗi service như một "nhân viên chuyên môn" làm 1 việc cụ thể
- Services giao tiếp với nhau qua API

---

## 🎯 Phục vụ vấn đề gì?

### Vấn đề với Monolith (Ứng dụng nguyên khối)
```
❌ Monolith Application:
┌─────────────────────────────────┐
│  Auth + Payment + User + Email  │
│  (Tất cả trong 1 codebase)      │
└─────────────────────────────────┘

Vấn đề:
- Sửa Auth → Phải deploy lại toàn bộ
- Auth bị lỗi → Toàn bộ app chết
- Team lớn → Conflict code liên tục
- Scale → Phải scale cả app (tốn tài nguyên)
```

### Giải pháp với Microservices
```
✅ Microservices:
┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐
│   Auth   │  │ Payment  │  │   User   │  │  Email   │
│ Service  │  │ Service  │  │ Service  │  │ Service  │
└──────────┘  └──────────┘  └──────────┘  └──────────┘

Lợi ích:
- Sửa Auth → Chỉ deploy Auth Service
- Auth lỗi → Các service khác vẫn hoạt động
- Mỗi team phụ trách 1 service → Không conflict
- Scale riêng từng service theo nhu cầu
```

---

## 🏗️ Kiến trúc tổng quan

```
┌─────────────────────────────────────────────────────────────┐
│                         CLIENTS                             │
│              Browser, Mobile App, Desktop                   │
└────────────────────────────┬────────────────────────────────┘
                             │ HTTP/HTTPS
                             ▼
┌─────────────────────────────────────────────────────────────┐
│                    HAProxy (Port 8080)                      │
│                    Load Balancer                            │
└────────────────────────────┬────────────────────────────────┘
                             │
                ┌────────────┼────────────┐
                │            │            │
                ▼            ▼            ▼
        ┌──────────┐  ┌──────────┐  ┌──────────┐
        │ Gateway  │  │ Gateway  │  │ Gateway  │
        │Instance 1│  │Instance 2│  │Instance 3│
        │Port 8083 │  │Port 8083 │  │Port 8083 │
        └────┬─────┘  └────┬─────┘  └────┬─────┘
             │             │             │
             └─────────────┼─────────────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
        ▼                  ▼                  ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│ Auth Service │  │ Core Service │  │Worker Service│
│  Port 8081   │  │  Port 8082   │  │(Background)  │
└──────┬───────┘  └──────┬───────┘  └──────┬───────┘
       │                 │                  │
       └─────────────────┼──────────────────┘
                         │
        ┌────────────────┼────────────────┐
        │                │                │
        ▼                ▼                ▼
┌──────────────┐  ┌──────────┐  ┌──────────┐
│  PostgreSQL  │  │  Redis   │  │  Kafka   │
│  Port 5432   │  │Port 6379 │  │Port 9092 │
└──────────────┘  └──────────┘  └──────────┘
```

---

## 📦 Cấu trúc Source Code

### 1. Crates (Shared Libraries)

**Crates** là các thư viện dùng chung giữa các services:

| Crate | Vai trò | Ví dụ |
|-------|---------|-------|
| `common` | Config, errors, cache, HTTP client | Load config, Redis cache, error handling |
| `db` | Database connection pool | Kết nối PostgreSQL, query helpers |
| `contracts` | DTOs, events (shared types) | `User`, `Payment`, `LoginRequest` |
| `authz` | JWT authentication/authorization | Verify JWT, extract user claims |
| `messaging` | Kafka producer/consumer | Publish/subscribe events |

**Lợi ích:**
- Tránh duplicate code giữa các services
- Đảm bảo consistency (cùng 1 cách xử lý lỗi, config)
- Dễ maintain: sửa 1 chỗ, tất cả services được update

### 2. Services (Microservices)

| Service | Port | Vai trò | Endpoints chính |
|---------|------|---------|-----------------|
| `gateway` | 8080 | API Gateway, rate limiting, routing | `/api/*` (proxy to other services) |
| `auth-service` | 8081 | Login, register, JWT | `/api/v1/auth/login`, `/api/v1/auth/register` |
| `core-service` | 8082 | Business logic, user management | `/api/v1/users`, `/api/v1/payments` |
| `worker-service` | - | Background jobs, Kafka consumers | (Không có HTTP endpoint) |

**Workflow:**
```
1. Client → Gateway (8080)
2. Gateway → Auth Service (8081) để verify JWT
3. Gateway → Core Service (8082) để lấy data
4. Core Service → Kafka → Worker Service xử lý background
```

### 3. Infrastructure

| Component | Vai trò | Khi nào dùng |
|-----------|---------|--------------|
| **PostgreSQL** | Database chính | Lưu trữ users, payments, orders |
| **Redis** | Cache + rate limiting | Cache data, đếm requests per user |
| **Kafka** | Event streaming | Gửi email, notifications, analytics |
| **HAProxy** | Load balancing | Phân phối traffic đến nhiều Gateway instances |

---

## 🔄 Communication Patterns

### 1. Synchronous (HTTP) - Đồng bộ

**Khi nào dùng:** Cần response ngay lập tức

```
Client → Gateway → Auth Service (verify JWT)
                → Core Service (get user data)
                → Response ngay
```

**Ví dụ:**
```bash
# Client gọi Gateway
GET /api/v1/users/123
Authorization: Bearer <JWT>

# Gateway:
1. Gọi Auth Service verify JWT
2. Gọi Core Service lấy user data
3. Trả response về client
```

### 2. Asynchronous (Kafka) - Bất đồng bộ

**Khi nào dùng:** Không cần response ngay, xử lý background

```
Gateway → Kafka → Worker Service (send email)
                → Analytics Service (log event)
                → Notification Service (push notification)
```

**Ví dụ:**
```rust
// Gateway publish event
kafka_producer.send("user.registered", UserRegisteredEvent {
    user_id: 123,
    email: "user@example.com"
});

// Worker Service subscribe và xử lý
kafka_consumer.subscribe(&["user.registered"]);
// → Gửi welcome email
// → Tạo user profile
// → Log analytics
```

---

## 📊 So sánh Sync vs Async

| Tiêu chí | Synchronous (HTTP) | Asynchronous (Kafka) |
|----------|-------------------|----------------------|
| **Response** | Ngay lập tức | Không có (fire and forget) |
| **Use case** | Login, get data | Send email, analytics |
| **Coupling** | Tight (phải chờ response) | Loose (không cần biết consumer) |
| **Failure** | Client biết ngay | Client không biết |
| **Example** | `GET /users/123` | `user.registered` event |

---

## ✅ Khi nào nên dùng Microservices?

| Tình huống | Nên dùng? | Lý do |
|------------|-----------|-------|
| Large team (10+ developers) | ✅ Có | Mỗi team phụ trách 1 service, không conflict |
| Different scaling needs | ✅ Có | Scale riêng service cần thiết (VD: Auth scale nhiều hơn Payment) |
| Independent deployment | ✅ Có | Deploy Auth không ảnh hưởng Payment |
| Polyglot persistence | ✅ Có | Auth dùng PostgreSQL, Analytics dùng MongoDB |
| Small projects (1-3 devs) | ❌ Không | Overhead cao, phức tạp không cần thiết |
| Tight coupling giữa services | ❌ Không | Nếu services phụ thuộc lẫn nhau → Monolith tốt hơn |

---

## 🎯 Ưu điểm & Nhược điểm

### ✅ Ưu điểm
- **Scalability**: Scale riêng từng service theo nhu cầu
- **Resilience**: 1 service lỗi không ảnh hưởng toàn bộ
- **Team autonomy**: Mỗi team độc lập, deploy riêng
- **Technology diversity**: Mỗi service dùng tech stack phù hợp

### ❌ Nhược điểm
- **Complexity**: Nhiều services → khó debug, monitor
- **Network latency**: Gọi qua network chậm hơn in-process
- **Data consistency**: Khó đảm bảo ACID transactions giữa services
- **Overhead**: Cần infrastructure (Kafka, Redis, HAProxy)

---

## 💡 Tóm tắt

**Microservices Architecture** chia ứng dụng thành nhiều services nhỏ, độc lập:
- **Mục đích**: Scalability, resilience, team autonomy
- **Cách hoạt động**: Services giao tiếp qua HTTP (sync) và Kafka (async)
- **Vai trò**: Gateway routing, Auth xác thực, Core business logic, Worker background jobs
- **Kết quả**: Dễ scale, deploy độc lập, nhưng phức tạp hơn monolith
