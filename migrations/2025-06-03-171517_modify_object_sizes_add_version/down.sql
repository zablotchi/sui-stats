-- This file should undo anything in `up.sql`

-- Drop the new indexes
DROP INDEX IF EXISTS idx_object_sizes_version;
DROP INDEX IF EXISTS idx_object_sizes_cp_version;

-- Drop the new primary key
ALTER TABLE object_sizes DROP CONSTRAINT object_sizes_pkey;

-- Remove the version column
ALTER TABLE object_sizes DROP COLUMN version;

-- Recreate the original primary key
ALTER TABLE object_sizes ADD PRIMARY KEY (object_id, cp_sequence_number, is_input);

-- Recreate the original index
CREATE INDEX idx_object_sizes_cp ON object_sizes (cp_sequence_number);
