-- Add usage_metadata to ai_messages and ai_conversations
ALTER TABLE ai_messages ADD COLUMN usage_metadata TEXT;
ALTER TABLE ai_conversations ADD COLUMN usage_metadata TEXT;
