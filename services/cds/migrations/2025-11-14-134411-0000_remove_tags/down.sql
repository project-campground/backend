ALTER TABLE appview.profile_post
ADD COLUMN IF NOT EXISTS tags text array NOT NULL
DEFAULT '{}';