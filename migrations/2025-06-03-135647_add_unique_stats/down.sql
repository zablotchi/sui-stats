-- This file should undo anything in `up.sql`

ALTER TABLE sizes DROP COLUMN unique_object_ids;
ALTER TABLE sizes DROP COLUMN unique_event_ids;
