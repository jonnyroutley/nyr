CREATE TABLE progress_records (
    id INTEGER PRIMARY KEY,
    target_id INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    entry_date DATE DEFAULT CURRENT_DATE,
    value REAL,
    FOREIGN KEY(target_id) REFERENCES targets(id)
)
