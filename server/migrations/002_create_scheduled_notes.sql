CREATE TABLE IF NOT EXISTS scheduled_notes (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  text TEXT NOT NULL,
  body_json TEXT NOT NULL,
  scheduled_at TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'pending',
  substack_id TEXT,
  substack_url TEXT,
  error TEXT,
  attempts INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_notes_due ON scheduled_notes(scheduled_at, status);
CREATE INDEX IF NOT EXISTS idx_notes_user ON scheduled_notes(user_id, status);
