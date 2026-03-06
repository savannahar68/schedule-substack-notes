CREATE TABLE IF NOT EXISTS users (
  id TEXT PRIMARY KEY,
  encrypted_cookies TEXT NOT NULL,
  cookie_iv TEXT NOT NULL,
  auth_token TEXT UNIQUE NOT NULL,
  substack_handle TEXT,
  cookies_valid_at TEXT,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
