mod app;
mod progress_bar;
mod progress_records;
mod targets;

use std::str::FromStr;

use clap::{Parser, Subcommand};
use iocraft::prelude::*;
use sqlx::{
    migrate::{MigrateDatabase, Migrator},
    Sqlite, SqlitePool,
};

use directories::ProjectDirs;
use include_dir::{include_dir, Dir};
use std::path::PathBuf;

static MIGRATIONS_DIR: Dir = include_dir!("./migrations");

fn get_db_path() -> Result<(String, PathBuf), Box<dyn std::error::Error>> {
    let proj_dirs =
        ProjectDirs::from("", "", "nyr").ok_or("Failed to determine project directories")?;

    let data_dir = proj_dirs.data_dir();
    std::fs::create_dir_all(data_dir)?;

    let db_path = data_dir.join("database.sqlite");

    let db_url = format!("sqlite:{}", db_path.display());

    let migrations_path = data_dir.join("migrations");
    if !migrations_path.exists() {
        std::fs::create_dir_all(&migrations_path)?;
        // Extract bundled migrations
        for entry in MIGRATIONS_DIR.files() {
            std::fs::write(
                migrations_path.join(entry.path().file_name().unwrap()),
                entry.contents(),
            )?;
        }
    }

    Ok((db_url, migrations_path))
}

async fn ensure_db_and_tables_exist() -> Result<SqlitePool, Box<dyn std::error::Error>> {
    let (db_url, migrations_path) = get_db_path()?;

    if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
        log::debug!("Creating database {}", db_url);
        Sqlite::create_database(&db_url).await.map_err(|error| {
            log::error!("Failed to create database: {}", error);
            error
        })?;
        log::debug!("Create db success");
    } else {
        log::debug!("Database already exists");
    }

    let db = SqlitePool::connect(&db_url).await.map_err(|error| {
        log::error!("Failed to connect to database: {}", error);
        error
    })?;

    match Migrator::new(migrations_path).await?.run(&db).await {
        Ok(_) => log::debug!("Migration successful"),
        Err(error) => {
            log::error!("Migration failed: {}", error);
            return Err(error.into());
        }
    }

    Ok(db)
}

#[derive(Parser)]
#[command(name = "progress")]
#[command(author = "Jonathan Routley <jonathan.wei.liang@gmail.com>")]
#[command(version = "1.0")]
#[command(about = "A tool to manage progress tracking", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Targets {
        #[command(subcommand)]
        action: TargetCommands,
    },
    Records {
        #[command(subcommand)]
        action: RecordCommands,
    },
}

#[derive(Subcommand)]
enum TargetCommands {
    List,
    Create {
        #[arg(short, long)]
        /// The target you're trying to achieve.
        name: String,

        #[arg(long)]
        /// (Optional) The type of target ("count" or "value") you're trying to achieve. Defaults to "count".
        target_type: Option<String>,

        #[arg(short = 'd', long)]
        /// (Optional) When you'd like to achieve the goal by. Defaults to end of this year.
        target_date: Option<chrono::NaiveDate>,

        #[arg(short, long)]
        /// (Optional) The starting value of your target. Defaults to 0.
        start_value: Option<f64>,

        #[arg(short, long)]
        /// The target value you're trying to achieve.
        target_value: f64,
    },
    Delete {
        #[arg(short, long)]
        id: i64,
    },
}

#[derive(Subcommand)]
enum RecordCommands {
    List,
    Create {
        #[arg(short, long)]
        /// The id of the target that this record is for.
        target_id: i64,
        #[arg(short, long)]
        /// (Optional) When the record was done. Defaults to today.
        entry_date: Option<chrono::NaiveDate>,
        #[arg(short, long)]
        /// (Optional for "value" targets) The name of the record.
        item_name: Option<String>,
        #[arg(short, long)]
        /// (Optional for "count" targets) The value you want to record.
        value: Option<f64>,
    },
    Delete {
        #[arg(short, long)]
        id: i64,
    },
}

#[tokio::main]
async fn main() {
    let db = ensure_db_and_tables_exist().await.unwrap();
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::Targets { action }) => match action {
            TargetCommands::List => {
                let targets = targets::get_targets(&db).await;
                element!(targets::TargetsTable(targets: &targets, title: "targets")).print();
            }
            TargetCommands::Create {
                name,
                target_date,
                target_type,
                start_value,
                target_value,
            } => {
                let checked_target_type = match target_type {
                    Some(x) => targets::TargetType::from_str(x).unwrap(),
                    None => targets::TargetType::Count,
                };

                let target_create_result = targets::create_target(
                    &db,
                    name,
                    target_date,
                    checked_target_type,
                    start_value,
                    target_value,
                )
                .await;
                let mut targets = Vec::new();
                targets.push(target_create_result);
                element!(targets::TargetsTable(targets: &targets, title: "target created")).print();
            }
            TargetCommands::Delete { id } => {
                targets::delete_target(&db, id).await;
            }
        },
        Some(Commands::Records { action }) => match action {
            RecordCommands::List => {
                let progress_records = progress_records::get_progress_records(&db).await;
                element!(progress_records::ProgressRecordsTable(progress_records: &progress_records, title: "progress records")).print();
            }
            RecordCommands::Create {
                target_id,
                entry_date,
                item_name,
                value,
            } => {
                let target = targets::get_target(&db, &target_id).await;
                match target.target_type {
                    targets::TargetType::Count => {
                        if item_name.is_none() {
                            panic!("Item name is required for count targets");
                        }
                    }
                    targets::TargetType::Value => {
                        if value.is_none() {
                            panic!("Value is required for value targets");
                        }
                    }
                }

                let progress_record_create_result = progress_records::create_progress_record(
                    &db, &target_id, entry_date, value, item_name,
                )
                .await;
                let mut progress_records = Vec::new();
                progress_records.push(progress_record_create_result);
                element!(progress_records::ProgressRecordsTable(progress_records: &progress_records, title: "progress record created")).print();
            }
            RecordCommands::Delete { id } => {
                progress_records::delete_progress_record(&db, id).await;
                println!("Record deleted");
            }
        },
        None => {
            let target_progresses = targets::get_progress_for_all_targets(&db).await;
            app::run_app(target_progresses);
        }
    }
}
