# Kafka Event Streaming

## 🔍 Kafka Event Streaming là gì?

**Kafka** là hệ thống message queue (hàng đợi tin nhắn) cho phép các services giao tiếp với nhau bất đồng bộ thông qua events.

**Ví dụ đơn giản:**
- Giống như hệ thống thông báo trong công ty: người gửi đăng thông báo lên bảng tin, người quan tâm đọc
- Producer (người gửi) không cần biết ai sẽ đọc
- Consumer (người đọc) không cần biết ai đã gửi
- Decoupled (tách rời) hoàn toàn!

---

## 🎯 Phục vụ vấn đề gì?

### Vấn đề với Synchronous Communication

```
❌ Synchronous (HTTP):
┌──────────┐                    ┌──────────────┐
│ Gateway  │ ──── Request ────► │ Email Service│
│          │                    │              │
│          │ ◄─── Response ──── │              │
└──────────┘                    └──────────────┘
     │
     │ Phải chờ Email Service xong
     │ Email Service chậm → Gateway chậm
     │ Email Service lỗi → Gateway lỗi
     ▼
   Slow!

Vấn đề:
- Gateway phải chờ Email Service gửi email xong
- Email Service chậm (5s) → User chờ 5s
- Email Service lỗi → Request failed
- Tight coupling: Gateway phụ thuộc Email Service
```

### Giải pháp với Kafka (Asynchronous)

```
✅ Asynchronous (Kafka):
┌──────────┐                    ┌──────────┐
│ Gateway  │ ──── Event ──────► │  Kafka   │
│          │                    │  Topic   │
│          │ ◄─── OK (ngay) ─── │          │
└──────────┘                    └────┬─────┘
     │                               │
     │ Response ngay                 │ Event được lưu
     │ Không cần chờ                 │
     ▼                               ▼
   Fast!                    ┌──────────────┐
                            │Email Service │
                            │(xử lý async) │
                            └──────────────┘

Lợi ích:
- Gateway publish event và response ngay (< 10ms)
- Email Service xử lý background, không block Gateway
- Email Service lỗi → Retry tự động, không ảnh hưởng Gateway
- Loose coupling: Services độc lập
```

---

## 🏗️ Vai trò trong Source Code

### 1. **Event-Driven Architecture**
```
User registers → Gateway → Kafka → Multiple Consumers
                                    ├─► Email Service (send welcome email)
                                    ├─► Analytics Service (log event)
                                    ├─► Notification Service (push notification)
                                    └─► CRM Service (create lead)
```

### 2. **Decoupling Services**
- Gateway không cần biết có bao nhiêu consumers
- Thêm consumer mới không cần sửa Gateway
- Mỗi consumer xử lý độc lập

### 3. **Audit Log & Event Sourcing**
- Mọi event được lưu trong Kafka
- Có thể replay events để debug
- Audit trail đầy đủ

---

## 📋 Topics (Chủ đề)

| Topic | Mô tả | Producer | Consumers |
|-------|-------|----------|-----------|
| `payment.created` | Payment mới được tạo | Core Service | Email, Analytics, Notification |
| `payment.updated` | Payment status thay đổi | Core Service | Email, Analytics |
| `user.registered` | User mới đăng ký | Auth Service | Email, CRM, Analytics |
| `notification.email` | Gửi email | Gateway | Email Worker |

---

## 📖 Cách sử dụng

### 1. Producer (Publish Event)

**Gateway publish event:**
```rust
use messaging::KafkaProducer;
use contracts::events::PaymentCreatedEvent;

// Tạo event
let event = PaymentCreatedEvent {
    payment_id: "pi_123".to_string(),
    user_id: 456,
    amount: 10000,
    currency: "USD".to_string(),
    created_at: Utc::now(),
};

// Publish lên Kafka
kafka_producer
    .send("payment.created", &event)
    .await?;

// Response ngay cho user (không chờ consumer)
Ok(Json(PaymentResponse {
    id: "pi_123",
    status: "pending"
}))
```

### 2. Consumer (Subscribe & Process)

**Worker Service subscribe event:**
```rust
use messaging::KafkaConsumer;
use contracts::events::PaymentCreatedEvent;

// Subscribe topic
let consumer = KafkaConsumer::new()?;
consumer.subscribe(&["payment.created"])?;

// Lắng nghe và xử lý events
for message in consumer.iter() {
    // Deserialize event
    let event: PaymentCreatedEvent = 
        serde_json::from_slice(&message.payload)?;
    
    // Xử lý event
    match event {
        PaymentCreatedEvent { payment_id, user_id, amount, .. } => {
            // Gửi email confirmation
            email_service.send_payment_confirmation(
                user_id, 
                payment_id, 
                amount
            ).await?;
            
            // Log analytics
            analytics.track_payment(payment_id, amount).await?;
            
            // Commit offset (đánh dấu đã xử lý)
            consumer.commit_message(&message)?;
        }
    }
}
```

---

## 🔄 Workflow hoàn chỉnh

```
1. User tạo payment
   ↓
   POST /api/v1/payments
   ↓
   Gateway tạo payment trong DB
   ↓
   Gateway publish event "payment.created" lên Kafka
   ↓
   Response ngay cho user (200 OK)

2. Kafka lưu event
   ↓
   Event được lưu trong topic "payment.created"
   ↓
   Kafka broadcast event đến tất cả consumers

3. Consumers xử lý (parallel)
   ↓
   Email Service: Gửi email confirmation
   Analytics Service: Log payment event
   Notification Service: Push notification
   ↓
   Mỗi consumer xử lý độc lập, không block nhau
```

---

## 📊 So sánh Sync vs Async

| Tiêu chí | Synchronous (HTTP) | Asynchronous (Kafka) |
|----------|-------------------|----------------------|
| **Response time** | Chậm (phải chờ) | Nhanh (ngay lập tức) |
| **Coupling** | Tight (phụ thuộc lẫn nhau) | Loose (độc lập) |
| **Failure handling** | Request failed nếu service lỗi | Retry tự động, không ảnh hưởng producer |
| **Scalability** | Khó (phải scale cả chain) | Dễ (scale riêng từng consumer) |
| **Use case** | Login, get data | Email, notifications, analytics |

---

## ✅ Khi nào nên dùng Kafka?

| Tình huống | Nên dùng? | Lý do |
|------------|-----------|-------|
| Notifications (email, SMS, push) | ✅ Có | Không cần response ngay, xử lý background |
| Analytics và reporting | ✅ Có | Thu thập data từ nhiều sources |
| Audit logging | ✅ Có | Lưu trữ tất cả events, có thể replay |
| Background processing | ✅ Có | Image processing, video encoding, etc. |
| Microservices communication | ✅ Có | Decouple services, scale độc lập |
| Cần response ngay lập tức | ❌ Không | Dùng HTTP synchronous |
| Critical path operations | ❌ Không | Login, payment processing (dùng HTTP) |
| Simple CRUD operations | ❌ Không | Overhead không cần thiết |

---

## 🛠️ Event Examples

### Payment Created Event
```rust
#[derive(Serialize, Deserialize)]
pub struct PaymentCreatedEvent {
    pub payment_id: String,
    pub user_id: i64,
    pub amount: i64,
    pub currency: String,
    pub created_at: DateTime<Utc>,
}
```

### User Registered Event
```rust
#[derive(Serialize, Deserialize)]
pub struct UserRegisteredEvent {
    pub user_id: i64,
    pub email: String,
    pub name: String,
    pub registered_at: DateTime<Utc>,
}
```

### Email Notification Event
```rust
#[derive(Serialize, Deserialize)]
pub struct EmailNotificationEvent {
    pub to: String,
    pub subject: String,
    pub body: String,
    pub template: Option<String>,
}
```

---

## 🔧 Configuration

### Producer Config
```rust
// Gateway - Kafka Producer
let producer = KafkaProducer::new(KafkaConfig {
    brokers: vec!["localhost:9092".to_string()],
    client_id: "gateway-producer".to_string(),
    acks: "all",  // Wait for all replicas
    retries: 3,
})?;
```

### Consumer Config
```rust
// Worker Service - Kafka Consumer
let consumer = KafkaConsumer::new(KafkaConfig {
    brokers: vec!["localhost:9092".to_string()],
    group_id: "email-worker-group".to_string(),
    topics: vec!["payment.created", "user.registered"],
    auto_offset_reset: "earliest",  // Read from beginning
})?;
```

---

## 🚨 Error Handling

### Producer Error
```rust
// Retry logic
match kafka_producer.send("payment.created", &event).await {
    Ok(_) => info!("Event published successfully"),
    Err(e) => {
        error!("Failed to publish event: {}", e);
        // Fallback: Lưu vào DB để retry sau
        db.save_failed_event(&event).await?;
    }
}
```

### Consumer Error
```rust
// Xử lý lỗi và retry
for message in consumer.iter() {
    match process_event(&message).await {
        Ok(_) => {
            // Success: Commit offset
            consumer.commit_message(&message)?;
        }
        Err(e) => {
            error!("Failed to process event: {}", e);
            // Retry hoặc gửi vào dead letter queue
            dead_letter_queue.send(&message).await?;
        }
    }
}
```

---

## 💡 Tóm tắt

**Kafka Event Streaming** cho phép services giao tiếp bất đồng bộ:
- **Mục đích**: Decouple services, xử lý background tasks
- **Cách hoạt động**: Producer publish events, consumers subscribe và xử lý
- **Vai trò**: Notifications, analytics, audit logging, background processing
- **Kết quả**: Response nhanh, services độc lập, dễ scale, fault-tolerant
