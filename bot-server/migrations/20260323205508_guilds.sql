-- Add migration script here

CREATE TABLE servers (
	agent_id UUID NOT NULL UNIQUE,
	guild_id BIGINT NOT NULL,
	chat_channel_id BIGINT,
	query_channel_id BIGINT,
	query_message_id BIGINT
)
