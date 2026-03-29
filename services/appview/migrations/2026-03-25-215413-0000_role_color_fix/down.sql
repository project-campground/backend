ALTER TABLE appview.campsite_role
ADD COLUMN IF NOT EXISTS color integer NOT NULL
DEFAULT 0;

ALTER TABLE appview.campsite_role
ADD COLUMN IF NOT EXISTS colorsecondary integer NOT NULL
DEFAULT 0;

UPDATE appview.campsite_role SET color = COALESCE(colors[1]::Integer, 0), colorsecondary = COALESCE(colors[2]::Integer, 0);

ALTER TABLE appview.campsite_role
DROP COLUMN colors;

ALTER TABLE appview.campsite_role
DROP COLUMN motion;