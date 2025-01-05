CREATE TABLE targets (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    target_date DATE,
    status TEXT CHECK (status IN ('active', 'completed', 'abandoned')) DEFAULT 'active',
    target_type TEXT CHECK (target_type IN ('count', 'value')) NOT NULL,
    start_value REAL,
    target_value REAL,
    count_goal INTEGER,
    value_goal REAL,
    CHECK ((target_type = 'count' AND count_goal IS NOT NULL AND value_goal IS NULL) OR 
          (target_type = 'value' AND value_goal IS NOT NULL AND count_goal IS NULL))
);
