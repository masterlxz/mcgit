CREATE TABLE IF NOT EXISTS java_installations (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    major_version INTEGER NOT NULL,
    vendor        TEXT NOT NULL,
    path          TEXT NOT NULL UNIQUE,
    source        TEXT NOT NULL CHECK (source IN ('managed', 'detected', 'manual')),
    is_default    INTEGER NOT NULL DEFAULT 0 CHECK (is_default IN (0, 1)),
    created_at    TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Enforces "at most one default" at the database level: SQLite refuses a second
-- row with is_default = 1, instead of relying on application code to remember.
CREATE UNIQUE INDEX IF NOT EXISTS idx_java_installations_default
    ON java_installations(is_default)
    WHERE is_default = 1;
