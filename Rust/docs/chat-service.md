# Chat Service - Tổng quan hệ thống

## 1. Giới thiệu

Chat Service là hệ thống nhắn tin real-time hỗ trợ:
- ✅ Chat 1-1 (tin nhắn riêng tư)
- ✅ Chat nhóm (group chat)
- ✅ Bảo mật với JWT
- ✅ Scale ngang với nhiều instances
- ✅ Lưu trữ lịch sử tin nhắn

## 2. Kiến trúc tổng quan

```
┌─────────────────────────────────────────────────────────────┐
│                    CLIENTS (Người dùng)                     │
│  Browser App    Mobile App    Desktop App    Web App        │
└──────────────────────────┬──────────────────────────────────┘
                           │ WebSocket (wss://)
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                   HAProxy Load Balancer                     │
│                      Port: 8085                             │
│              (Phân phối kết nối đến các instance)           │
└──────────────────────────┬──────────────────────────────────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
        ▼                  ▼                  ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│ Chat Service 1  │ │ Chat Service 2  │ │ Chat Service 3  │
│   Instance      │ │   Instance      │ │   Instance      │
│   Port 8084     │ │   Port 8084     │ │   Port 8084     │
└────────┬────────┘ └────────┬────────┘ └────────┬────────┘
         │                   │                   │
         └───────────────────┼───────────────────┘
                             │
         ┌───────────────────┼───────────────────┐
         │                   │                   │
         ▼                   ▼                   ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│  Redis Pub/Sub  │ │     MySQL       │ │     Kafka       │
│  Đồng bộ msg    │ │  Lưu trữ data   │ │  Thông báo      │
│   Port 6379     │ │   Port 3306     │ │   Port 9092     │
└─────────────────┘ └─────────────────┘ └─────────────────┘
```

## 3. Cách hoạt động

### 3.1. Kết nối WebSocket

```
┌────────┐                                    ┌──────────────┐
│ Client │                                    │   HAProxy    │
└───┬────┘                                    └──────┬───────┘
    │                                                │
    │ 1. Yêu cầu kết nối WebSocket                   │
    │    ws://localhost:8085/api/ws                  │
    ├───────────────────────────────────────────────►│
    │                                                │
    │                                                │ 2. Chọn server
    │                                                │    dựa trên IP
    │                                                │
    │                                         ┌──────▼───────┐
    │                                         │ Chat Service │
    │                                         │  Instance 1  │
    │                                         └──────┬───────┘
    │                                                │
    │ 3. Kết nối WebSocket thành công                │
    │◄───────────────────────────────────────────────┤
    │                                                │
    │ 4. Gửi JWT token để xác thực                   │
    ├───────────────────────────────────────────────►│
    │                                                │
    │ 5. Xác thực thành công                         │
    │◄───────────────────────────────────────────────┤
    │                                                │
    │ 6. Sẵn sàng gửi/nhận tin nhắn                  │
```

### 3.2. Gửi và nhận tin nhắn

```
User A (Instance 1)                    User B (Instance 2)
       │                                     │
       │ 1. Gửi tin nhắn "Hello"             │
       ├──────────►┌──────────────┐          │
       │           │  Instance 1  │          │
       │           └──────┬───────┘          │
       │                  │                  │
       │                  │ 2. Lưu vào DB    │
       │                  ▼                  │
       │           ┌──────────────┐          │
       │           │    MySQL     │          │
       │           └──────────────┘          │
       │                  │                  │
       │                  │ 3. Publish       │
       │                  ▼                  │
       │           ┌──────────────┐          │
       │           │    Redis     │          │
       │           │   Pub/Sub    │          │
       │           └──────┬───────┘          │
       │                  │                  │
       │    ┌─────────────┼─────────────┐    │
       │    │             │             │    │
       │    ▼             ▼             ▼    │
       │ ┌────────┐  ┌────────┐  ┌────────┐  │
       │ │Instance│  │Instance│  │Instance│  │
       │ │   1    │  │   2    │  │   3    │  │
       │ └───┬────┘  └───┬────┘  └────────┘  │
       │     │           │                   │
       │ 4. Gửi lại      │ 5. Gửi đến        │
       │     │           │    User B         │
       ◄─────┘           └─────────────────► │
       │                                     │
   Nhận tin                            Nhận tin
   của mình                            từ User A
```

**Giải thích:**
1. User A gửi tin nhắn qua WebSocket đến Instance 1
2. Instance 1 lưu tin nhắn vào MySQL (để có lịch sử)
3. Instance 1 publish tin nhắn lên Redis Pub/Sub
4. Tất cả instances (1, 2, 3) nhận tin nhắn từ Redis
5. Instance 2 gửi tin nhắn đến User B qua WebSocket

## 4. Các loại phòng chat

### 4.1. Direct Chat (Chat 1-1)

```
┌──────────────────────┐
│   Direct Room        │
│   (Chat riêng tư)    │
├──────────────────────┤
│                      │
│  ┌────┐    ┌────┐    │
│  │User│◄──►│User│    │
│  │ A  │    │ B  │    │
│  └────┘    └────┘    │
│                      │
│  • Chỉ 2 người       │
│  • Tự động tạo       │
│  • Không cần tên     │
│  • Bảo mật cao       │
└──────────────────────┘
```

**Tạo Direct Chat:**
```bash
POST /api/rooms
{
  "room_type": "direct",
  "member_ids": [2]  # ID của người muốn chat
}
```

### 4.2. Group Chat (Chat nhóm)

```
┌──────────────────────┐
│    Group Room        │
│   (Chat nhóm)        │
├──────────────────────┤
│                      │
│  ┌────┐              │
│  │User│ (Admin)      │
│  │ A  │              │
│  └─┬──┘              │
│    │                 │
│    ├──► ┌────┐       │
│    │    │User│       │
│    │    │ B  │       │
│    │    └────┘       │
│    │                 │
│    ├──► ┌────┐       │
│    │    │User│       │
│    │    │ C  │       │
│    │    └────┘       │
│    │                 │
│    └──► ┌────┐       │
│         │User│       │
│         │ D  │       │
│         └────┘       │
│                      │
│  • Nhiều người       │
│  • Có tên nhóm       │
│  • Có admin          │
└──────────────────────┘
```

**Tạo Group Chat:**
```bash
POST /api/rooms
{
  "name": "Team Chat",
  "room_type": "group",
  "member_ids": [2, 3, 4]  # Danh sách thành viên
}
```

## 5. Bảo mật

### 5.1. Các lớp bảo mật

```
┌─────────────────────────────────────────────────────────────┐
│                      Lớp bảo mật                            │
└─────────────────────────────────────────────────────────────┘

Lớp 1: Mã hóa kết nối
┌─────────────────────────────────────────────────────────────┐
│  WSS (WebSocket Secure) - TLS/SSL Encryption                │
└─────────────────────────────────────────────────────────────┘
                           │
Lớp 2: Xác thực           ▼
┌─────────────────────────────────────────────────────────────┐
│  JWT Token Validation                                       │
│  • Kiểm tra chữ ký                                          │
│  • Kiểm tra hết hạn                                         │
│  • Lấy user_id                                              │
└─────────────────────────────────────────────────────────────┘
                           │
Lớp 3: Phân quyền         ▼
┌─────────────────────────────────────────────────────────────┐
│  Room Membership Check                                      │
│  • Kiểm tra user có trong phòng không                       │
│  • Kiểm tra quyền của user                                  │
│  • Xác thực quyền truy cập                                  │
└─────────────────────────────────────────────────────────────┘
                           │
Lớp 4: Giới hạn tốc độ    ▼
┌─────────────────────────────────────────────────────────────┐
│  Rate Limiting                                              │
│  • Giới hạn số tin nhắn/giây                                │
│  • Ngăn spam                                                │
│  • Chặn người dùng lạm dụng                                 │
└─────────────────────────────────────────────────────────────┘
```

### 5.2. Bảo vệ tin nhắn riêng tư

**Kịch bản:** User C cố đọc tin nhắn giữa User A và User B

```
User C → Yêu cầu đọc tin nhắn của phòng A-B
         ↓
    Kiểm tra: User C có trong phòng không?
         ↓
    Kết quả: KHÔNG
         ↓
    Phản hồi: ❌ 403 Forbidden "Not a member of this room"
```

**User C KHÔNG THỂ:**
- ❌ Đọc tin nhắn của phòng A-B
- ❌ Tham gia WebSocket của phòng A-B
- ❌ Gửi tin nhắn vào phòng A-B
- ❌ Xem lịch sử tin nhắn của A-B

## 6. Cơ sở dữ liệu

```
┌─────────────────────────────────────────────────────────────┐
│                      chat_rooms                             │
│  (Bảng lưu thông tin phòng chat)                            │
├──────────────┬──────────────┬───────────────────────────────┤
│ id           │ VARCHAR(36)  │ Mã phòng (UUID)               │
│ name         │ VARCHAR(255) │ Tên phòng (có thể null)       │
│ room_type    │ VARCHAR(20)  │ 'direct' hoặc 'group'         │
│ created_by   │ BIGINT       │ ID người tạo                  │
│ created_at   │ TIMESTAMP    │ Thời gian tạo                 │
└──────────────┴──────────────┴───────────────────────────────┘
                       │
                       │ 1 phòng có nhiều thành viên
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                  chat_room_members                          │
│  (Bảng lưu thành viên của phòng)                            │
├──────────────┬──────────────┬───────────────────────────────┤
│ id           │ BIGINT       │ Mã tự động tăng               │
│ room_id      │ VARCHAR(36)  │ Mã phòng                      │
│ user_id      │ BIGINT       │ ID người dùng                 │
│ role         │ VARCHAR(20)  │ 'admin' hoặc 'member'         │
│ joined_at    │ TIMESTAMP    │ Thời gian tham gia            │
└──────────────┴──────────────┴───────────────────────────────┘
                       │
                       │ 1 phòng có nhiều tin nhắn
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                    chat_messages                            │
│  (Bảng lưu tin nhắn)                                        │
├──────────────┬──────────────┬───────────────────────────────┤
│ id           │ VARCHAR(36)  │ Mã tin nhắn (UUID)            │
│ room_id      │ VARCHAR(36)  │ Mã phòng                      │
│ sender_id    │ BIGINT       │ ID người gửi                  │
│ content      │ TEXT         │ Nội dung tin nhắn             │
│ message_type │ VARCHAR(20)  │ 'text','image','file'         │
│ created_at   │ TIMESTAMP    │ Thời gian gửi                 │
└──────────────┴──────────────┴───────────────────────────────┘
```

## 7. API và WebSocket

### 7.1. REST API

```bash
# Tạo phòng chat
POST /api/rooms
Authorization: Bearer <JWT_TOKEN>
Body: {"room_type": "direct", "member_ids": [2]}

# Lấy danh sách phòng của user
GET /api/rooms
Authorization: Bearer <JWT_TOKEN>

# Lấy lịch sử tin nhắn
GET /api/rooms/{room_id}/messages?limit=50
Authorization: Bearer <JWT_TOKEN>
```

### 7.2. WebSocket Messages

```javascript
// Kết nối
const ws = new WebSocket('ws://localhost:8085/api/ws');

// Tham gia phòng
ws.send(JSON.stringify({
  type: 'join_room',
  room_id: 'room-uuid'
}));

// Gửi tin nhắn
ws.send(JSON.stringify({
  type: 'message',
  room_id: 'room-uuid',
  content: 'Hello!',
  message_type: 'text'
}));

// Nhận tin nhắn
ws.onmessage = (event) => {
  const msg = JSON.parse(event.data);
  console.log(msg.content);
};
```

## 8. Triển khai (Deployment)

### 8.1. Cấu trúc Docker

```
┌─────────────────────────────────────────────────────────────┐
│                    Docker Compose                           │
└─────────────────────────────────────────────────────────────┘

┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│ chat-service │  │ chat-service │  │ chat-service │
│      -1      │  │      -2      │  │      -3      │
└──────┬───────┘  └──────┬───────┘  └──────┬───────┘
       │                 │                 │
       └─────────────────┼─────────────────┘
                         │
        ┌────────────────┼────────────────┐
        │                │                │
        ▼                ▼                ▼
┌───────────────┐ ┌──────────────┐ ┌──────────────┐
│    HAProxy    │ │    Redis     │ │    MySQL     │
│   Port 8085   │ │  Port 6379   │ │  Port 3306   │
└───────────────┘ └──────────────┘ └──────────────┘
```

### 8.2. Lệnh triển khai

```bash
# 1. Chạy database migrations
mysql -u root rustdb < migrations/chat_tables.sql

# 2. Khởi động services
cd Rust/infra
docker-compose up -d redis mysql
docker-compose up -d chat-service-1 chat-service-2 chat-service-3
docker-compose up -d haproxy

# 3. Kiểm tra health
curl http://localhost:8085/api/health
```

## 9. Giám sát (Monitoring)

### 9.1. HAProxy Stats

```
URL: http://localhost:8404

Hiển thị:
- Số kết nối đang hoạt động
- Tốc độ request
- Trạng thái health check
- Thời gian phản hồi
```

### 9.2. Logs

```bash
# Xem logs của tất cả chat services
docker-compose logs -f chat-service-1 chat-service-2 chat-service-3

# Xem logs của service cụ thể
docker-compose logs -f chat-service-1

# Lọc lỗi
docker-compose logs -f chat-service-1 | grep ERROR
```

## 10. Hiệu năng

### 10.1. Chỉ số hiệu năng

```
┌─────────────────────────────────────────────────────────────┐
│                    Performance Metrics                      │
└─────────────────────────────────────────────────────────────┘

Kết nối đồng thời:     10,000+ connections/instance
Độ trễ tin nhắn:       50-100ms
Throughput:            1,000+ messages/second/instance
Thời gian phản hồi:    < 100ms (REST API)
```

### 10.2. Tối ưu hóa

```
1. Connection Layer
   • WebSocket persistent connections
   • Connection pooling
   • Heartbeat mechanism

2. Application Layer
   • Async/await (non-blocking I/O)
   • Actor model (Actix)
   • Efficient JSON serialization

3. Caching Layer
   • Redis cho room membership
   • In-memory session cache
   • Message history cache

4. Database Layer
   • Indexed queries
   • Connection pooling
   • Read replicas
```

## 11. Xử lý lỗi

### 11.1. Instance bị lỗi

```
Trước khi lỗi:
┌────────────┐  ┌────────────┐  ┌────────────┐
│ Instance 1 │  │ Instance 2 │  │ Instance 3 │
│     ✓      │  │     ✓      │  │     ✓     │
└────────────┘  └────────────┘  └────────────┘

Sau khi lỗi:
┌────────────┐  ┌────────────┐  ┌────────────┐
│ Instance 1 │  │ Instance 2 │  │ Instance 3 │
│     ✓      │  │     ✓      │  │     ✗     │
└────────────┘  └────────────┘  └────────────┘

Kết quả:
• HAProxy tự động chuyển traffic sang Instance 1 và 2
• Users trên Instance 3 tự động reconnect
• Không mất tin nhắn (đã lưu trong MySQL)
```

### 11.2. Redis bị lỗi

```
Ảnh hưởng:
• Không đồng bộ tin nhắn giữa các instances
• Tin nhắn vẫn hoạt động trong cùng instance

Giải pháp:
• Tin nhắn vẫn được lưu vào MySQL
• Tự động reconnect Redis
• Alert monitoring team
```

## 12. Tóm tắt

### Ưu điểm:
✅ Real-time messaging với WebSocket
✅ Bảo mật cao với JWT và room membership
✅ Scale ngang với nhiều instances
✅ Lưu trữ lịch sử tin nhắn
✅ Hỗ trợ chat 1-1 và chat nhóm
✅ High availability với load balancing

### Công nghệ sử dụng:
- **Backend**: Rust + Actix Web
- **WebSocket**: Actix WebSocket
- **Database**: MySQL
- **Cache/Pub-Sub**: Redis
- **Load Balancer**: HAProxy
- **Container**: Docker + Docker Compose

### Endpoints chính:
- `GET /api/health` - Health check
- `POST /api/rooms` - Tạo phòng chat
- `GET /api/rooms` - Danh sách phòng
- `GET /api/rooms/{id}/messages` - Lịch sử tin nhắn
- `WS /api/ws` - WebSocket connection

---

**Tài liệu chi tiết:**
- `chat-architecture-diagram.md` - Sơ đồ kiến trúc chi tiết
- `chat-docker-deployment.md` - Hướng dẫn triển khai Docker
- `chat-security.md` - Bảo mật và phân quyền
- `chat-rate-limiting.md` - Giới hạn tốc độ
- `chat-service.md` - API documentation đầy đủ
