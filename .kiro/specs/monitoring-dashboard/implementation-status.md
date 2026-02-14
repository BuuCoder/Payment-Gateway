# Monitoring Dashboard - Implementation Status

## ✅ Phase 1: Database Schema (COMPLETED)
- ✅ Created migration file: `monitoring_metrics.sql`
- ✅ Tables created:
  - `metrics_system` - System metrics (memory, CPU, connections)
  - `metrics_connections` - WebSocket connection tracking
  - `metrics_messages` - Message count aggregated hourly
  - `metrics_rate_limits` - Rate limit violations
  - `metrics_user_activity` - User activity aggregated hourly
  - `metrics_alerts` - Suspicious activity alerts
- ✅ Views created:
  - `view_top_users_7d` - Top 20 users by message count
  - `view_system_summary_24h` - System metrics summary
  - `view_message_stats` - Message statistics by day

**Next Step**: Run migration
```sql
-- Run in MySQL/phpMyAdmin
SOURCE monitoring_metrics.sql;
```

## 🚧 Phase 2: Backend Metrics Collection (IN PROGRESS)

### 2.1 Metrics Collector Module
- [ ] Create `metrics` crate in `Rust/crates/metrics/`
- [ ] Implement `MetricsCollector` struct
- [ ] System metrics collection (memory, CPU, Redis)
- [ ] WebSocket metrics collection
- [ ] Rate limit violation tracking

### 2.2 Background Tasks
- [ ] Add to ChatServer: Collect metrics every 1 minute
- [ ] Add to ChatServer: Aggregate user activity every 5 minutes
- [ ] Add to ChatServer: Cleanup old metrics (> 1 year) daily

### 2.3 API Endpoints
- [ ] `GET /api/metrics/system` - Current system metrics
- [ ] `GET /api/metrics/connections?period=1d|7d|30d|1y` - Connection history
- [ ] `GET /api/metrics/messages?period=1d|7d|30d|1y` - Message history
- [ ] `GET /api/metrics/top-users?limit=20&period=1d|7d|30d` - Top active users
- [ ] `GET /api/metrics/alerts` - Active alerts
- [ ] `GET /api/metrics/summary` - Dashboard summary

## 📋 Phase 3: Frontend Dashboard (TODO)

### 3.1 Dashboard Page
- [ ] Create `/dashboard` page in Next.js
- [ ] Add authentication check (admin only)
- [ ] Layout with sidebar navigation

### 3.2 Components
- [ ] `SystemMetricsCards` - Memory, Redis, CPU, Connections
- [ ] `ConnectionsChart` - Line chart for connections over time
- [ ] `MessagesChart` - Bar chart for messages over time
- [ ] `TopUsersTable` - Table with top 20 users
- [ ] `AlertsPanel` - List of active alerts
- [ ] `PeriodSelector` - Dropdown to select time period

### 3.3 API Integration
- [ ] Create API client functions in `lib/api.ts`
- [ ] Add auto-refresh every 30 seconds
- [ ] Add loading states
- [ ] Add error handling

### 3.4 Charts Library
- [ ] Install Recharts: `npm install recharts`
- [ ] Create reusable chart components

## 🎯 Phase 4: Alerting (OPTIONAL)

### 4.1 Alert Detection
- [ ] Detect suspicious activity (>1000 messages/hour)
- [ ] Detect rate limit abuse (>10 violations/hour)
- [ ] Detect unusual connection patterns
- [ ] Store alerts in `metrics_alerts` table

### 4.2 Alert Notifications
- [ ] Email notifications (optional)
- [ ] Slack/Discord webhooks (optional)
- [ ] In-dashboard notifications

### 4.3 Auto-actions
- [ ] Auto-ban users with critical violations
- [ ] Auto-throttle suspicious IPs

## Current Progress: 10%

### Completed:
- ✅ Requirements document
- ✅ Database schema design
- ✅ Migration file created

### Next Steps:
1. Run database migration
2. Create metrics collector module
3. Add metrics collection to ChatServer
4. Create API endpoints
5. Build dashboard UI

## Estimated Time:
- Phase 2 (Backend): ~4 hours
- Phase 3 (Frontend): ~3 hours
- Phase 4 (Alerting): ~2 hours
- **Total**: ~9 hours

## Notes:
- Keep it simple for now (hệ thống còn nhỏ)
- Can scale later with Grafana/Prometheus if needed
- Focus on essential metrics first
- Add more metrics as needed
