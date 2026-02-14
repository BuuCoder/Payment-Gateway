# Monitoring Dashboard - Requirements

## Mục tiêu
Tạo dashboard để monitoring hệ thống real-time, phát hiện tấn công và tối ưu tài nguyên.

## Metrics cần thu thập

### 1. System Metrics (Real-time)
- **Memory Usage**: RAM đang dùng (MB/GB)
- **Redis Memory**: Dung lượng Redis đang dùng
- **CPU Usage**: % CPU usage
- **Active WebSocket Connections**: Số connections đang active
- **Active Users**: Số users đang online

### 2. Traffic Metrics (Time-series)
- **WebSocket Connections**: Số connections theo thời gian
  - 1 ngày (hourly)
  - 7 ngày (daily)
  - 30 ngày (daily)
  - 1 năm (monthly)
- **Messages Sent**: Số messages theo thời gian
  - Tổng messages
  - Messages per hour/day/week/month
- **API Requests**: Số HTTP requests

### 3. User Activity Metrics
- **Top 20 Active Users**: Users gửi nhiều messages nhất
  - User ID, Name, Email
  - Message count
  - Last active time
  - Rate limit violations (nếu có)
- **Suspicious Activity Detection**:
  - Users với rate limit violations > 10/day
  - Users gửi > 1000 messages/day
  - Users với nhiều connections đồng thời

### 4. Rate Limit Metrics
- **Rate Limit Violations**: Số lần bị rate limit
  - By event type (message, typing, room_action)
  - By user
  - By time period
- **Blocked Messages**: Số messages bị chặn

### 5. Database Metrics
- **Total Messages**: Tổng số messages trong DB
- **Total Users**: Tổng số users
- **Total Rooms**: Tổng số rooms
- **Database Size**: Kích thước database

## Architecture

### Backend (Rust)
1. **Metrics Collector Service**:
   - Thu thập metrics từ Redis, Database, System
   - Store metrics vào TimescaleDB hoặc InfluxDB (hoặc đơn giản: MySQL với timestamp)
   - Expose API endpoint `/api/metrics`

2. **Metrics Storage**:
   - Table: `metrics_websocket_connections`
   - Table: `metrics_messages`
   - Table: `metrics_rate_limits`
   - Table: `metrics_system`

3. **Background Tasks**:
   - Mỗi 1 phút: Thu thập system metrics
   - Mỗi 5 phút: Aggregate user activity
   - Mỗi 1 giờ: Cleanup old metrics (> 1 năm)

### Frontend (Next.js)
1. **Dashboard Page**: `/dashboard`
2. **Components**:
   - System Overview Cards
   - Time-series Charts (Chart.js hoặc Recharts)
   - Top Users Table
   - Alerts Panel
3. **Auto-refresh**: Mỗi 30 giây

## Database Schema

```sql
-- System metrics (real-time snapshot)
CREATE TABLE metrics_system (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    timestamp DATETIME NOT NULL,
    memory_used_mb INT,
    redis_memory_mb INT,
    cpu_usage_percent DECIMAL(5,2),
    active_connections INT,
    active_users INT,
    INDEX idx_timestamp (timestamp)
);

-- WebSocket connection metrics
CREATE TABLE metrics_connections (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    timestamp DATETIME NOT NULL,
    connection_count INT,
    new_connections INT,
    closed_connections INT,
    INDEX idx_timestamp (timestamp)
);

-- Message metrics
CREATE TABLE metrics_messages (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    timestamp DATETIME NOT NULL,
    message_count INT,
    user_id BIGINT,
    room_id VARCHAR(36),
    INDEX idx_timestamp (timestamp),
    INDEX idx_user_id (user_id)
);

-- Rate limit violations
CREATE TABLE metrics_rate_limits (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    timestamp DATETIME NOT NULL,
    user_id BIGINT NOT NULL,
    event_type VARCHAR(50),
    violation_count INT DEFAULT 1,
    INDEX idx_timestamp (timestamp),
    INDEX idx_user_id (user_id)
);

-- User activity summary (aggregated hourly)
CREATE TABLE metrics_user_activity (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    user_id BIGINT NOT NULL,
    hour_timestamp DATETIME NOT NULL,
    message_count INT DEFAULT 0,
    typing_count INT DEFAULT 0,
    room_action_count INT DEFAULT 0,
    rate_limit_violations INT DEFAULT 0,
    UNIQUE KEY unique_user_hour (user_id, hour_timestamp),
    INDEX idx_user_id (user_id),
    INDEX idx_timestamp (hour_timestamp)
);
```

## API Endpoints

### GET /api/metrics/system
Response:
```json
{
  "memory_used_mb": 512,
  "redis_memory_mb": 128,
  "cpu_usage_percent": 45.2,
  "active_connections": 150,
  "active_users": 120,
  "timestamp": "2026-02-14T10:30:00Z"
}
```

### GET /api/metrics/connections?period=1d|7d|30d|1y
Response:
```json
{
  "period": "7d",
  "data": [
    {"timestamp": "2026-02-14T00:00:00Z", "count": 150},
    {"timestamp": "2026-02-14T01:00:00Z", "count": 120}
  ]
}
```

### GET /api/metrics/messages?period=1d|7d|30d|1y
Response:
```json
{
  "period": "7d",
  "total": 15000,
  "data": [
    {"timestamp": "2026-02-14T00:00:00Z", "count": 500},
    {"timestamp": "2026-02-14T01:00:00Z", "count": 450}
  ]
}
```

### GET /api/metrics/top-users?limit=20&period=1d|7d|30d
Response:
```json
{
  "period": "7d",
  "users": [
    {
      "user_id": 2,
      "name": "Majin",
      "email": "majin@example.com",
      "message_count": 1500,
      "rate_limit_violations": 5,
      "last_active": "2026-02-14T10:25:00Z",
      "is_suspicious": false
    }
  ]
}
```

### GET /api/metrics/alerts
Response:
```json
{
  "alerts": [
    {
      "type": "suspicious_activity",
      "user_id": 5,
      "user_name": "Spammer",
      "message": "User sent 5000 messages in 1 hour",
      "severity": "high",
      "timestamp": "2026-02-14T10:00:00Z"
    },
    {
      "type": "rate_limit_abuse",
      "user_id": 8,
      "user_name": "Bot",
      "message": "50 rate limit violations in 1 hour",
      "severity": "critical",
      "timestamp": "2026-02-14T09:30:00Z"
    }
  ]
}
```

## Implementation Plan

### Phase 1: Database Schema ✅
- Create migration file
- Add metrics tables

### Phase 2: Metrics Collection (Rust)
- Add metrics collector to ChatServer
- Background task to collect system metrics
- Store metrics to database
- Expose metrics API endpoints

### Phase 3: Dashboard Frontend
- Create dashboard page
- Add charts and visualizations
- Add auto-refresh
- Add alerts panel

### Phase 4: Alerting (Optional)
- Email alerts for suspicious activity
- Slack/Discord webhooks
- Auto-ban users with critical violations

## UI Design

```
┌─────────────────────────────────────────────────────────────┐
│  📊 System Monitoring Dashboard                             │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │ Memory   │  │ Redis    │  │ Active   │  │ Messages │   │
│  │ 512 MB   │  │ 128 MB   │  │ 150      │  │ 15.2K    │   │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘   │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │ WebSocket Connections (Last 7 days)                    │ │
│  │ [Line Chart]                                            │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │ Messages Sent (Last 7 days)                            │ │
│  │ [Bar Chart]                                             │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │ Top 20 Active Users                                     │ │
│  │ ┌──────┬────────┬───────┬──────────┬─────────────────┐ │ │
│  │ │ Rank │ Name   │ Email │ Messages │ Rate Violations │ │ │
│  │ ├──────┼────────┼───────┼──────────┼─────────────────┤ │ │
│  │ │  1   │ Majin  │ maj.. │  1,500   │       5         │ │ │
│  │ │  2   │ User2  │ use.. │  1,200   │       2         │ │ │
│  │ └──────┴────────┴───────┴──────────┴─────────────────┘ │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │ 🚨 Alerts                                               │ │
│  │ • User "Spammer" sent 5000 messages in 1 hour          │ │
│  │ • User "Bot" has 50 rate limit violations              │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Security
- Dashboard chỉ accessible bởi admin users
- Add authentication middleware
- Rate limit cho dashboard API
- Không expose sensitive user data

## Performance
- Cache metrics trong Redis (TTL 30s)
- Aggregate old data (> 7 days) thành hourly/daily
- Delete metrics > 1 year
- Use indexes cho queries

## Benefits
1. **Phát hiện tấn công**: Nhận biết spam/bot ngay lập tức
2. **Tối ưu tài nguyên**: Biết khi nào cần scale
3. **User insights**: Hiểu user behavior
4. **Troubleshooting**: Debug issues nhanh hơn
5. **Compliance**: Audit trail cho security
