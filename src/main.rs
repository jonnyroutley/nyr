mod targets;
mod progress_records;

use chrono::Datelike;
use clap::{ Parser, Subcommand };
use iocraft::prelude::*;
use sqlx::{ migrate::MigrateDatabase, Sqlite, SqlitePool };

const DB_URL: &str = "sqlite://sqlite.db";

async fn ensure_db_and_tables_exist() -> sqlx::Pool<Sqlite> {
    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        log::debug!("Creating database {}", DB_URL);
        match Sqlite::create_database(DB_URL).await {
            Ok(_) => log::debug!("Create db success"),
            Err(error) => panic!("error: {}", error),
        }
    } else {
        log::debug!("Database already exists");
    }

    let db = SqlitePool::connect(DB_URL).await.unwrap();

    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let migrations = std::path::Path::new(&crate_dir).join("./migrations");

    let migration_results = sqlx::migrate::Migrator::new(migrations).await.unwrap().run(&db).await;

    match migration_results {
        Ok(_) => {}
        // println!("Migration success"),
        Err(error) => {
            panic!("error: {}", error);
        }
    }

    log::debug!("migration: {:?}", migration_results);
    db
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
        target_id: String,
        #[arg(short, long)]
        /// (Optional) When the record was done. Defaults to today.
        entry_date: Option<chrono::NaiveDate>,
        #[arg(short, long)]
        /// The value you want to record.
        value: f64,
    },
    Delete {
        #[arg(short, long)]
        id: i64,
    },
}

#[tokio::main]
async fn main() {
    let db = ensure_db_and_tables_exist().await;
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::Targets { action }) =>
            match action {
                TargetCommands::List => {
                    let targets = sqlx
                        ::query_as::<_, targets::Target>("SELECT * FROM targets")
                        .fetch_all(&db).await
                        .unwrap();
                    element!(targets::TargetsTable(targets: &targets, title: "targets")).print();
                }
                TargetCommands::Create { name, target_date, start_value, target_value } => {
                    let last_date_this_year = chrono::NaiveDate
                        ::from_ymd_opt(chrono::Utc::now().year(), 12, 31)
                        .unwrap();

                    let target_create_result = sqlx
                        ::query_as::<_, targets::Target>(
                            "INSERT INTO targets (name, target_date, status, start_value, target_value)
                        VALUES ($1, $2, $3, $4, $5)
                        RETURNING *;"
                        )
                        .bind(name)
                        .bind(match target_date {
                            Some(x) => x,
                            None => { &last_date_this_year }
                        })
                        .bind("active")
                        .bind(match start_value {
                            Some(x) => x,
                            None => &0.0,
                        })
                        .bind(target_value)
                        .fetch_one(&db).await
                        .unwrap();
                    let mut targets = Vec::new();
                    targets.push(target_create_result);
                    element!(targets::TargetsTable(targets: &targets, title: "target created")).print();
                }
                TargetCommands::Delete { id } => {
                    sqlx::query("DELETE FROM targets WHERE id=$1")
                        .bind(id)
                        .execute(&db).await
                        .unwrap();
                    println!("Target deleted");
                }
            }
        Some(Commands::Records { action }) =>
            match action {
                RecordCommands::List => {
                    println!("Listing all records");
                    let progress_records = sqlx
                        ::query_as::<_, progress_records::ProgressRecord>(
                            "SELECT * FROM progress_records"
                        )
                        .fetch_all(&db).await
                        .unwrap();
                    element!(progress_records::ProgressRecordsTable(progress_records: &progress_records, title: "progress records")).print();
                }
                RecordCommands::Create { target_id, entry_date, value } => {
                    let today = chrono::Utc::now().date_naive();
                    let progress_record_create_result = sqlx
                        ::query_as::<_, progress_records::ProgressRecord>(
                            "INSERT INTO progress_records (target_id, entry_date, value)
                    VALUES ($1, $2, $3)
                    RETURNING *;"
                        )
                        .bind(target_id)
                        .bind(match entry_date {
                            Some(x) => x,
                            None => { &today }
                        })
                        .bind(value)
                        .fetch_one(&db).await
                        .unwrap();
                    let mut progress_records = Vec::new();
                    progress_records.push(progress_record_create_result);
                    element!(progress_records::ProgressRecordsTable(progress_records: &progress_records, title: "progress record created")).print();
                }
                RecordCommands::Delete { id } => {
                    sqlx::query("DELETE FROM progress_records WHERE id=$1")
                        .bind(id)
                        .execute(&db).await
                        .unwrap();
                    println!("Record deleted");
                }
            }
        None => { println!("Welcome") }
    }
}
