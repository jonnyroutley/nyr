CREATE TABLE targets (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    target_date DATE,
    status TEXT CHECK (status IN ('active', 'completed', 'abandoned')) DEFAULT 'active',
    target_type TEXT CHECK (target_type IN ('count', 'value')) NOT NULL,
    start_value REAL,
    target_value REAL
);
