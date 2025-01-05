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

pub async fn get_progress_for_target(db: &Pool<Sqlite>, target_id: &i64) -> f64 {
    let target = get_target(db, target_id).await;
    let current: f64 = match target.target_type {
        TargetType::Count => sqlx::query_scalar!(
            "SELECT COUNT (*) from progress_records where target_id = ?",
            target_id,
        )
        .fetch_one(db)
        .await
        .unwrap() as f64,

        TargetType::Value => sqlx::query_scalar!(
            "SELECT MAX (value) from progress_records WHERE target_id = ?",
            target_id
        )
        .fetch_one(db)
        .await
        .unwrap()
        .unwrap_or(target.start_value),
    };

    current / target.target_value
}
