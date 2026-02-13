# Chat Application

Ứng dụng chat real-time được xây dựng với Next.js và kết nối với Rust backend services.

## Tính năng

- Đăng nhập / Đăng ký
- Tìm kiếm và kết bạn
- Chat 1-1 (Direct Message)
- Chat real-time qua WebSocket
- Lịch sử tin nhắn

## Cài đặt

```bash
npm install
```

## Cấu hình

Tạo file `.env.local` với các biến môi trường:

```
NEXT_PUBLIC_AUTH_API_URL=http://localhost:8081
NEXT_PUBLIC_CHAT_API_URL=http://localhost:8082
NEXT_PUBLIC_CHAT_WS_URL=ws://localhost:8082
NEXT_PUBLIC_CORE_API_URL=http://localhost:8083
```

## Chạy ứng dụng

Development mode:
```bash
npm run dev
```

Build production:
```bash
npm run build
npm start
```

## API Endpoints được sử dụng

### Auth Service (port 8081)
- POST `/api/v1/auth/register` - Đăng ký tài khoản
- POST `/api/v1/auth/login` - Đăng nhập

### Chat Service (port 8082)
- GET `/api/rooms` - Lấy danh sách phòng chat
- POST `/api/rooms/direct` - Tạo phòng chat 1-1
- GET `/api/rooms/{room_id}/messages` - Lấy tin nhắn trong phòng
- WS `/api/ws` - WebSocket connection cho chat real-time

### Core Service (port 8083)
- GET `/api/users` - Lấy danh sách người dùng (để tìm bạn)

## Cấu trúc thư mục

```
src/
├── app/
│   ├── layout.tsx          # Layout chính
│   ├── page.tsx            # Trang chủ (redirect)
│   ├── login/
│   │   └── page.tsx        # Trang đăng nhập/đăng ký
│   └── chat/
│       └── page.tsx        # Trang chat chính
├── lib/
│   ├── api.ts              # API client functions
│   └── websocket.ts        # WebSocket client
└── types/
    └── index.ts            # TypeScript types
```

## Hướng dẫn sử dụng

1. Đăng ký tài khoản mới hoặc đăng nhập
2. Click "Tìm bạn bè" để xem danh sách người dùng
3. Click vào người dùng để tạo phòng chat 1-1
4. Gửi tin nhắn real-time qua WebSocket
5. Xem lịch sử tin nhắn khi chọn phòng chat

## Lưu ý

- Đảm bảo các Rust services đang chạy trước khi khởi động ứng dụng
- Token JWT được lưu trong localStorage
- WebSocket tự động reconnect khi mất kết nối
