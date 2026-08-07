CREATE TABLE IF NOT EXISTS message_owners (
    chat_id INTEGER NOT NULL,
    message_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    quote_id INTEGER,
    PRIMARY KEY (chat_id, message_id)
);

ALTER TABLE quotes ADD COLUMN score INTEGER NOT NULL DEFAULT 0;
