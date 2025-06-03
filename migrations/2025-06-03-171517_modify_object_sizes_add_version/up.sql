-- Your SQL goes here

-- Add version column to object_sizes table
ALTER TABLE object_sizes ADD COLUMN version BIGINT NOT NULL DEFAULT 0;

-- Drop the existing primary key constraint
ALTER TABLE object_sizes DROP CONSTRAINT object_sizes_pkey;

-- Create new primary key with object_id and version
ALTER TABLE object_sizes ADD PRIMARY KEY (object_id, version);

-- Drop the old index and create new one for common query patterns
DROP INDEX IF EXISTS idx_object_sizes_cp;
CREATE INDEX idx_object_sizes_cp_version ON object_sizes (cp_sequence_number, version);
CREATE INDEX idx_object_sizes_version ON object_sizes (version);
