-- Your SQL goes here

ALTER TABLE sizes ADD COLUMN unique_object_ids BIGINT NOT NULL DEFAULT 0;
ALTER TABLE sizes ADD COLUMN unique_event_ids BIGINT NOT NULL DEFAULT 0;
