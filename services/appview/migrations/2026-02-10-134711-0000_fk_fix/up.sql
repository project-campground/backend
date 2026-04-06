ALTER TABLE appview.campsite_permission
ADD CONSTRAINT fk_bonfire
FOREIGN KEY (bonfireId) REFERENCES appview.bonfire(id)
ON DELETE CASCADE;