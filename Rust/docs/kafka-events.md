# Kafka Event Streaming

## Tác dụng
- Asynchronous communication giữa services
- Decouple services (không cần biết consumer)
- Event sourcing và audit log

## Topics
- `payment.created`: Payment mới được tạo
- `payment.updated`: Payment status thay đổi
- `user.registered`: User mới đăng ký
- `notification.email`: Gửi email

## Producer (Gateway)
```rust
// Publish event
kafka_producer.send(
    "payment.created",
    &PaymentCreatedEvent {
        payment_id,
        user_id,
        amount,
    }
).await?;
```

## Consumer (Worker Service)
```rust
// Subscribe và xử lý
kafka_consumer.subscribe(&["payment.created"])?;

for message in kafka_consumer.iter() {
    let event: PaymentCreatedEvent = serde_json::from_slice(&message)?;
    // Process event (send email, update analytics, etc.)
}
```

## Khi nào dùng
- ✅ Notifications (email, SMS)
- ✅ Analytics và reporting
- ✅ Audit logging
- ✅ Background processing
- ❌ Cần response ngay (dùng HTTP)
- ❌ Critical path operations
