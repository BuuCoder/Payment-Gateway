-- Chat Enhancements Migration
-- Date: 2026-02-14
-- Description: Add support for invitations, room sorting, leave/hide, and unread tracking

-- ============================================
-- 1. Create room_invitations table
-- ============================================
CREATE TABLE IF NOT EXISTS room_invitations (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    room_id VARCHAR(36) NOT NULL,
    user_id BIGINT NOT NULL,
    invited_by BIGINT NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- 'pending', 'accepted', 'declined'
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (room_id) REFERENCES chat_rooms(id) ON DELETE CASCADE,
    UNIQUE KEY unique_room_user_invitation (room_id, user_id),
    INDEX idx_user_status (user_id, status),
    INDEX idx_room_id (room_id),
    INDEX idx_status (status),
    INDEX idx_created_at (created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================
-- 2. Add last_message_at to chat_rooms
-- ============================================
ALTER TABLE chat_rooms
ADD COLUMN last_message_at TIMESTAMP NULL AFTER updated_at,
ADD INDEX idx_last_message_at (last_message_at);

-- ============================================
-- 3. Add columns to chat_room_members
-- ============================================
ALTER TABLE chat_room_members
ADD COLUMN left_at TIMESTAMP NULL AFTER joined_at,
ADD COLUMN hidden_at TIMESTAMP NULL AFTER left_at,
ADD COLUMN last_read_at TIMESTAMP NULL DEFAULT CURRENT_TIMESTAMP AFTER hidden_at,
ADD INDEX idx_left_at (left_at),
ADD INDEX idx_hidden_at (hidden_at),
ADD INDEX idx_last_read_at (last_read_at);

-- ============================================
-- 4. Backfill last_message_at from existing messages
-- ============================================
UPDATE chat_rooms r
SET last_message_at = (
    SELECT MAX(created_at)
    FROM chat_messages m
    WHERE m.room_id = r.id
)
WHERE EXISTS (
    SELECT 1
    FROM chat_messages m
    WHERE m.room_id = r.id
);

-- ============================================
-- 5. Create indexes for performance optimization
-- ============================================

-- Index for unread count calculation
CREATE INDEX idx_messages_room_created_sender 
ON chat_messages(room_id, created_at, sender_id);

-- Composite index for room listing with filters
CREATE INDEX idx_members_user_left_hidden 
ON chat_room_members(user_id, left_at, hidden_at);

-- Index for invitation queries
CREATE INDEX idx_invitations_user_created 
ON room_invitations(user_id, created_at DESC);

