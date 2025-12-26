-- Add msg_type and target_id to chat_messages table
ALTER TABLE chat_messages ADD COLUMN msg_type TEXT DEFAULT 'chat';
ALTER TABLE chat_messages ADD COLUMN target_id TEXT;
