-- Monitoring Metrics Tables
-- Run this migration to create metrics tables

-- System metrics (real-time snapshot every minute)
CREATE TABLE IF NOT EXISTS metrics_system (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    memory_used_mb INT,
    redis_memory_mb INT,
    cpu_usage_percent DECIMAL(5,2),
    active_connections INT,
    active_users INT,
    INDEX idx_timestamp (timestamp)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- WebSocket connection metrics (every minute)
CREATE TABLE IF NOT EXISTS metrics_connections (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    connection_count INT,
    new_connections INT DEFAULT 0,
    closed_connections INT DEFAULT 0,
    INDEX idx_timestamp (timestamp)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Message metrics (aggregated hourly)
CREATE TABLE IF NOT EXISTS metrics_messages (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    hour_timestamp DATETIME NOT NULL,
    message_count INT DEFAULT 0,
    unique_users INT DEFAULT 0,
    unique_rooms INT DEFAULT 0,
    UNIQUE KEY unique_hour (hour_timestamp),
    INDEX idx_timestamp (hour_timestamp)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Rate limit violations (every occurrence)
CREATE TABLE IF NOT EXISTS metrics_rate_limits (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    user_id BIGINT NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    violation_count INT DEFAULT 1,
    INDEX idx_timestamp (timestamp),
    INDEX idx_user_id (user_id),
    INDEX idx_event_type (event_type)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- User activity summary (aggregated hourly)
CREATE TABLE IF NOT EXISTS metrics_user_activity (
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
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Alerts table (for suspicious activity)
CREATE TABLE IF NOT EXISTS metrics_alerts (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    alert_type VARCHAR(50) NOT NULL,
    severity ENUM('low', 'medium', 'high', 'critical') NOT NULL,
    user_id BIGINT,
    message TEXT,
    is_resolved BOOLEAN DEFAULT FALSE,
    resolved_at DATETIME,
    INDEX idx_timestamp (timestamp),
    INDEX idx_user_id (user_id),
    INDEX idx_severity (severity),
    INDEX idx_resolved (is_resolved)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Create views for easy querying

-- View: Top users by message count (last 7 days)
CREATE OR REPLACE VIEW view_top_users_7d AS
SELECT 
    u.id as user_id,
    u.name,
    u.email,
    SUM(mua.message_count) as total_messages,
    SUM(mua.rate_limit_violations) as total_violations,
    MAX(u.last_seen) as last_active,
    CASE 
        WHEN SUM(mua.message_count) > 1000 THEN TRUE
        WHEN SUM(mua.rate_limit_violations) > 10 THEN TRUE
        ELSE FALSE
    END as is_suspicious
FROM users u
LEFT JOIN metrics_user_activity mua ON u.id = mua.user_id
WHERE mua.hour_timestamp >= DATE_SUB(NOW(), INTERVAL 7 DAY)
GROUP BY u.id, u.name, u.email
ORDER BY total_messages DESC
LIMIT 20;

-- View: System metrics summary (last 24 hours)
CREATE OR REPLACE VIEW view_system_summary_24h AS
SELECT 
    AVG(memory_used_mb) as avg_memory_mb,
    MAX(memory_used_mb) as max_memory_mb,
    AVG(redis_memory_mb) as avg_redis_mb,
    MAX(redis_memory_mb) as max_redis_mb,
    AVG(cpu_usage_percent) as avg_cpu_percent,
    MAX(cpu_usage_percent) as max_cpu_percent,
    AVG(active_connections) as avg_connections,
    MAX(active_connections) as max_connections,
    AVG(active_users) as avg_users,
    MAX(active_users) as max_users
FROM metrics_system
WHERE timestamp >= DATE_SUB(NOW(), INTERVAL 24 HOUR);

-- View: Message stats by period
CREATE OR REPLACE VIEW view_message_stats AS
SELECT 
    DATE(hour_timestamp) as date,
    SUM(message_count) as daily_messages,
    AVG(unique_users) as avg_unique_users,
    AVG(unique_rooms) as avg_unique_rooms
FROM metrics_messages
WHERE hour_timestamp >= DATE_SUB(NOW(), INTERVAL 30 DAY)
GROUP BY DATE(hour_timestamp)
ORDER BY date DESC;
