-- Add sender_id to chat_messages table to track message sender for DM grouping
ALTER TABLE chat_messages ADD COLUMN sender_id VARCHAR(255);
