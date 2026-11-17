CREATE INDEX IF NOT EXISTS campsite_invite_campsite_id_idx
    ON appview.campsite_invite (campsiteId);
CREATE INDEX IF NOT EXISTS tent_message_tent_id_idx
    ON appview.tent_message (tentId);
CREATE INDEX IF NOT EXISTS campsite_vanity_idx
    ON appview.campsite (vanityUrl);

ALTER TABLE appview.campsite_member
ALTER COLUMN usedInviteId TYPE uuid USING NULL;

-- Campsite foreign key
ALTER TABLE appview.campsite_member
ADD CONSTRAINT fk_campsite
FOREIGN KEY (campsiteId) REFERENCES appview.campsite(id)
ON DELETE CASCADE;

ALTER TABLE appview.campsite_invite
ADD CONSTRAINT fk_campsite
FOREIGN KEY (campsiteId) REFERENCES appview.campsite(id)
ON DELETE CASCADE;

ALTER TABLE appview.campsite_ban
ADD CONSTRAINT fk_campsite
FOREIGN KEY (campsiteId) REFERENCES appview.campsite(id)
ON DELETE CASCADE;

ALTER TABLE appview.campsite_permission
ADD CONSTRAINT fk_campsite
FOREIGN KEY (campsiteId) REFERENCES appview.campsite(id)
ON DELETE CASCADE;

ALTER TABLE appview.campsite_role
ADD CONSTRAINT fk_campsite
FOREIGN KEY (campsiteId) REFERENCES appview.campsite(id)
ON DELETE CASCADE;

ALTER TABLE appview.bonfire
ADD CONSTRAINT fk_campsite
FOREIGN KEY (campsiteId) REFERENCES appview.campsite(id)
ON DELETE CASCADE;

ALTER TABLE appview.tent_category
ADD CONSTRAINT fk_campsite
FOREIGN KEY (campsiteId) REFERENCES appview.campsite(id)
ON DELETE CASCADE;

ALTER TABLE appview.tent
ADD CONSTRAINT fk_campsite
FOREIGN KEY (campsiteId) REFERENCES appview.campsite(id)
ON DELETE CASCADE;

ALTER TABLE appview.tent_message
ADD CONSTRAINT fk_tent
FOREIGN KEY (tentId) REFERENCES appview.tent(id)
ON DELETE CASCADE;

ALTER TABLE appview.campsite_permission
ADD CONSTRAINT fk_tent
FOREIGN KEY (tentId) REFERENCES appview.tent(id)
ON DELETE CASCADE;

ALTER TABLE appview.campsite_permission
ADD CONSTRAINT fk_category
FOREIGN KEY (categoryId) REFERENCES appview.tent_category(id)
ON DELETE CASCADE;