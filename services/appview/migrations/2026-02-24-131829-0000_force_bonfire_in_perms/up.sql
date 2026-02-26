DELETE FROM appview.campsite_permission WHERE bonfireId IS NULL;
ALTER TABLE appview.campsite_permission ALTER COLUMN bonfireId SET NOT NULL;

CREATE INDEX IF NOT EXISTS campsite_permission_bonfire_id_idx
    ON appview.campsite_permission (bonfireId);

WITH existing_bonfires AS (SELECT array_agg(id) AS id FROM appview.bonfire)
DELETE FROM appview.tent WHERE NOT bonfireId = ANY((SELECT id FROM existing_bonfires)::Text[]);

ALTER TABLE appview.tent
ADD CONSTRAINT fk_bonfire
FOREIGN KEY (bonfireId) REFERENCES appview.bonfire(id)
ON DELETE CASCADE;