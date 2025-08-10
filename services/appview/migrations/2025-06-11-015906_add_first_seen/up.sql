-- Add first seen to Profile
ALTER TABLE appview.profile
ADD COLUMN IF NOT EXISTS firstSeen character varying NOT NULL;
