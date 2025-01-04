use sqlx::FromRow;

#[derive(Clone, FromRow, Debug)]
pub struct Record {
    id: i64,
    target_id: i64,
    created_at: chrono::NaiveDateTime,
    entry_date: chrono::NaiveDate,
    value: f64,
}