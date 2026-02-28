ALTER TABLE appview.tent_message
ADD COLUMN IF NOT EXISTS components jsonb array NOT NULL
DEFAULT '{}';
ALTER TABLE appview.tent_message
ADD COLUMN IF NOT EXISTS type smallint NOT NULL
DEFAULT 0;

-- Alter tent type
ALTER TABLE appview.tent
ALTER COLUMN type TYPE smallint USING type::smallint;
ALTER TABLE appview.tent
ALTER COLUMN viewType TYPE smallint USING viewType::smallint;