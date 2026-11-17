ALTER TABLE appview.campsite_role
ADD COLUMN IF NOT EXISTS colors integer array NOT NULL
DEFAULT '{}';

ALTER TABLE appview.campsite_role
ADD COLUMN IF NOT EXISTS motion smallint NOT NULL
DEFAULT 0;

UPDATE appview.campsite_role SET colors = array_remove(ARRAY[color, colorsecondary], 0);

ALTER TABLE appview.campsite_role
DROP COLUMN color;

ALTER TABLE appview.campsite_role
DROP COLUMN colorsecondary;