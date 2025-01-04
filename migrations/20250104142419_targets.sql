CREATE TABLE targets (
    id INTEGER PRIMARY KEY,
    name TEXT not NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    target_date DATE,
    status TEXT CHECK (status IN ('active', 'completed', 'abandoned')) DEFAULT 'active',
    start_value REAL DEFAULT 0.0,
    target_value REAL
)
