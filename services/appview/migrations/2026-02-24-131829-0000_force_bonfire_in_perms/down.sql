ALTER TABLE appview.campsite_permission ALTER COLUMN bonfireId DROP NOT NULL;

UPDATE appview.campsite_permission SET bonfireId = NULL WHERE categoryId IS NOT NULL OR tentId IS NOT NULL;

ALTER TABLE appview.tent
DROP CONSTRAINT IF EXISTS fk_bonfire;

DROP INDEX IF EXISTS campsite_permission_bonfire_id_idx;