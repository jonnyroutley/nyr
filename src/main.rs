use chrono::Datelike;
use clap::{ Parser, Subcommand };
use iocraft::prelude::*;
use sqlx::{ migrate::MigrateDatabase, FromRow, Sqlite, SqlitePool };

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

#[derive(Clone, FromRow, Debug)]
struct Target {
    id: i64,
    name: String,
    target_date: chrono::NaiveDate,
    status: String,
    start_value: f64,
    target_value: f64,
}

#[derive(Clone, FromRow, Debug)]
struct ProgressRecord {
    id: i64,
    target_id: i64,
    created_at: chrono::NaiveDateTime,
    entry_date: chrono::NaiveDate,
    value: f64,
}

#[derive(Default, Props)]
struct TargetsTableProps<'a> {
    targets: Option<&'a Vec<Target>>,
    title: &'a str,
}

#[component]
fn TargetsTable<'a>(props: &TargetsTableProps<'a>) -> impl Into<AnyElement<'a>> {
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

                View(width: 40pct, justify_content: JustifyContent::Center) {
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

                    View(width: 40pct, justify_content: JustifyContent::Center) {
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
        name: String,
        #[arg(short = 'd', long)]
        target_date: chrono::NaiveDate,
        #[arg(short, long)]
        start_value: f64,
        #[arg(short, long)]
        target_value: f64,
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
                        ::query_as::<_, Target>("SELECT * FROM targets")
                        .fetch_all(&db).await
                        .unwrap();
                    element!(TargetsTable(targets: &targets, title: "targets")).print();
                }
                TargetCommands::Create { name, target_date, start_value, target_value } => {
                    let last_date_this_year = chrono::NaiveDate
                        ::from_ymd_opt(chrono::Utc::now().year(), 12, 31)
                        .unwrap();

                    let target_create_result = sqlx
                        ::query_as::<_, Target>(
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
                    element!(TargetsTable(targets: &targets, title: "target created")).print();
                }
                TargetCommands::Delete { id } => {
                    println!("Deleting record with ID: {}", id);
                    sqlx::query("DELETE FROM targets WHERE id=$1")
                        .bind(id)
                        .execute(&db).await
                        .unwrap();
                }
            }
        Some(Commands::Records { action }) =>
            match action {
                RecordCommands::List => {
                    println!("Listing all records");
                }
                RecordCommands::Create { name, target_date, start_value, target_value } => {
                    println!("Creating a new record with name: {}", name);
                }
                RecordCommands::Delete { id } => {
                    println!("Deleting record with ID: {}", id);
                }
            }
        None => { println!("Welcome") }
    }
}
