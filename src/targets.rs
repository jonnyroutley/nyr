use std::str::FromStr;

use chrono::{Datelike, NaiveDate};
use iocraft::prelude::*;
use sqlx::{FromRow, Pool, Sqlite};

#[derive(Clone, Debug, sqlx::Type)]
#[sqlx(type_name = "target_type", rename_all = "lowercase")]
pub enum TargetType {
    Count,
    Value,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseTargetTypeError;

impl FromStr for TargetType {
    type Err = ParseTargetTypeError;

    fn from_str(input: &str) -> Result<TargetType, Self::Err> {
        match input {
            "count" => Ok(TargetType::Count),
            "value" => Ok(TargetType::Value),
            _ => Err(ParseTargetTypeError),
        }
    }
}

#[derive(Clone, FromRow, Debug)]
pub struct Target {
    pub id: i64,
    pub name: String,
    pub target_date: chrono::NaiveDate,
    pub status: String,
    pub start_value: f64,
    pub target_value: f64,
    pub target_type: TargetType,
}

#[derive(Default, Props)]
pub struct TargetsTableProps<'a> {
    pub targets: Option<&'a Vec<Target>>,
    pub title: &'a str,
}

#[component]
pub fn TargetsTable<'a>(props: &TargetsTableProps<'a>) -> impl Into<AnyElement<'a>> {
    element! {
        View(
            margin_top: 1,
            margin_bottom: 1,
            flex_direction: FlexDirection::Column,
            width: 100,
            border_style: BorderStyle::Round,
            border_color: Color::Cyan,
        ) {
            View(width: 100pct, justify_content: JustifyContent::Center, margin_bottom:1, ) {
                Text(content: props.title, weight: Weight::Bold )
            }

            View(border_style: BorderStyle::Single, border_edges: Edges::Bottom, border_color: Color::Grey) {
                View(width: 10pct, justify_content: JustifyContent::Center) {
                    Text(content: "id", weight: Weight::Bold, decoration: TextDecoration::Underline)
                }

                View(width: 25pct, justify_content: JustifyContent::Center) {
                    Text(content: "name", weight: Weight::Bold, decoration: TextDecoration::Underline)
                }

                View(width: 12.5pct, justify_content: JustifyContent::Center) {
                    Text(content: "target date", weight: Weight::Bold, decoration: TextDecoration::Underline)
                }
                View(width: 12.5pct, justify_content: JustifyContent::Center) {
                    Text(content: "status", weight: Weight::Bold, decoration: TextDecoration::Underline)
                }
                View(width: 12.5pct, justify_content: JustifyContent::Center) {
                    Text(content: "start", weight: Weight::Bold, decoration: TextDecoration::Underline)
                }
                View(width: 12.5pct, justify_content: JustifyContent::Center) {
                    Text(content: "target", weight: Weight::Bold, decoration: TextDecoration::Underline)
                }
            }

            #(props.targets.map(|targets| targets.iter().enumerate().map(|(i, target)| element! {
                View(background_color: if i % 2 == 0 { None } else { Some(Color::DarkGrey) }) {
                    View(width: 10pct, justify_content: JustifyContent::Center) {
                        Text(content: target.id.to_string())
                    }

                    View(width: 25pct, justify_content: JustifyContent::Center) {
                        Text(content: target.name.clone())
                    }

                    View(width: 12.5pct, justify_content: JustifyContent::Center) {
                        Text(content: target.target_date.to_string())
                    }
                    View(width: 12.5pct, justify_content: JustifyContent::Center) {
                        Text(content: target.status.to_string())
                    }
                    View(width: 12.5pct, justify_content: JustifyContent::Center) {
                        Text(content: target.start_value.to_string())
                    }
                    View(width: 12.5pct, justify_content: JustifyContent::Center) {
                        Text(content: target.target_value.to_string())
                    }
                }
            })).into_iter().flatten())
        }
    }
}

pub async fn get_targets(db: &Pool<Sqlite>) -> Vec<Target> {
    sqlx::query_as::<_, Target>("SELECT * FROM targets")
        .fetch_all(db)
        .await
        .unwrap()
}

pub async fn get_target(db: &Pool<Sqlite>, id: &i64) -> Target {
    sqlx::query_as::<_, Target>("SELECT * FROM targets WHERE id=$1")
        .bind(id)
        .fetch_one(db)
        .await
        .unwrap()
}

pub async fn create_target(
    db: &Pool<Sqlite>,
    name: &String,
    target_date: &Option<NaiveDate>,
    target_type: TargetType,
    start_value: &Option<f64>,
    target_value: &f64,
) -> Target {
    let last_date_this_year =
        chrono::NaiveDate::from_ymd_opt(chrono::Utc::now().year(), 12, 31).unwrap();

    sqlx::query_as::<_, Target>(
        "INSERT INTO targets (name, target_date, status,target_type, start_value, target_value)
                        VALUES ($1, $2, $3, $4, $5, $6)
                        RETURNING *;",
    )
    .bind(name)
    .bind(match target_date {
        Some(x) => x,
        None => &last_date_this_year,
    })
    .bind("active")
    .bind(target_type)
    .bind(match start_value {
        Some(x) => x,
        None => &0.0,
    })
    .bind(target_value)
    .fetch_one(db)
    .await
    .unwrap()
}

pub async fn delete_target(db: &Pool<Sqlite>, id: &i64) {
    sqlx::query("DELETE FROM targets WHERE id=$1")
        .bind(id)
        .execute(db)
        .await
        .unwrap();
    println!("Target deleted");
}

#[derive(Debug)]
pub struct TargetProgress {
    pub target_id: i64,
    pub percentage: f64,
    pub name: String,
    pub target_value: f64,
}

pub async fn get_progress_for_all_targets(db: &Pool<Sqlite>) -> Vec<TargetProgress> {
    let rows = sqlx::query!(
        r#"
        WITH progress_values AS (
            SELECT 
                t.id as target_id,
                t.target_type as target_type,
                t.target_value as target_value,
                t.start_value as start_value,
                t.name as name,
                CASE 
                    WHEN t.target_type = 'Count' THEN CAST(COUNT(pr.id) AS FLOAT)
                    WHEN t.target_type = 'Value' THEN COALESCE(MAX(pr.value), t.start_value)
                    ELSE 0 
                END as current_value
            FROM targets t
            LEFT JOIN progress_records pr ON t.id = pr.target_id
            GROUP BY t.id
        )
        SELECT 
            target_id,
            CAST(current_value AS FLOAT) / CAST(target_value AS FLOAT) as percentage,
            name,
            target_value
        FROM progress_values
        "#
    )
    .fetch_all(db)
    .await
    .unwrap_or_default();

    rows.into_iter()
        .map(|row| TargetProgress {
            target_id: row.target_id.unwrap(),
            percentage: row.percentage.unwrap_or(0.0),
            name: row.name.unwrap(),
            target_value: row.target_value.unwrap(),
        })
        .collect()
}

pub async fn get_progress_for_target(db: &Pool<Sqlite>, target_id: i64) -> f64 {
    let result = sqlx::query!(
        r#"
        WITH progress_value AS (
            SELECT 
                t.target_type as target_type,
                t.target_value as target_value,
                t.start_value as start_value,
                CASE 
                    WHEN t.target_type = 'Count' THEN CAST(COUNT(pr.id) AS FLOAT)
                    WHEN t.target_type = 'Value' THEN COALESCE(MAX(pr.value), t.start_value)
                    ELSE 0 
                END as current_value
            FROM targets t
            LEFT JOIN progress_records pr ON t.id = pr.target_id
            WHERE t.id = ?
            GROUP BY t.id
        )
        SELECT CAST(current_value AS FLOAT) / CAST(target_value AS FLOAT) as percentage
        FROM progress_value
        "#,
        target_id
    )
    .fetch_optional(db)
    .await
    .unwrap_or_default();

    result.map(|r| r.percentage.unwrap_or(0.0)).unwrap_or(0.0)
}
