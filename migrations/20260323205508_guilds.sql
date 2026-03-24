-- Add migration script here

CREATE TABLE servers (
	guild_id BIGINT NOT_NULL,
	connection_id UUID,
	chat_channel_id BIGINT,
	query_channel_id BIGINT,
	query_message_id BIGINT
)
