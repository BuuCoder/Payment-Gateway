-- Add last_seen column to users table
ALTER TABLE users ADD COLUMN last_seen DATETIME DEFAULT NULL;

-- Create index for faster queries
CREATE INDEX idx_users_last_seen ON users(last_seen);
