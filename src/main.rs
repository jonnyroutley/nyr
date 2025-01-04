use chrono::Datelike;
use clap::{ Parser, Subcommand };
use iocraft::prelude::*;
use sqlx::{ migrate::MigrateDatabase, FromRow, Sqlite, SqlitePool };

const DB_URL: &str = "sqlite://sqlite.db";

async fn ensure_db_and_tables_exist() -> sqlx::Pool<Sqlite> {
    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        // println!("Creating database {}", DB_URL);
        match Sqlite::create_database(DB_URL).await {
            Ok(_) => println!("Create db success"),
            Err(error) => panic!("error: {}", error),
        }
    } else {
        // println!("Database already exists");
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

    // println!("migration: {:?}", migration_results);
    db
}
// fn main() {
//     element! {
//         View(
//             border_style: BorderStyle::Round,
//             border_color: Color::Blue,
//         ) {
//             Text(content: "Hello, Fraser!")
//         }
//     }
//     .print();
// }

#[derive(Clone, FromRow, Debug)]
struct Target {
    id: i64,
    name: String,
    created_at: chrono::NaiveDateTime,
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
                    let target_results = sqlx
                        ::query_as::<_, Target>("SELECT * FROM targets")
                        .fetch_all(&db).await
                        .unwrap();
                    println!("Listing all targets");
                    println!("{:?}", target_results)
                }
                TargetCommands::Create { name, target_date, start_value, target_value } => {
                    println!("Creating a new record");
                    let last_date_this_year = chrono::NaiveDate
                        ::from_ymd_opt(chrono::Utc::now().year(), 12, 31)
                        .unwrap();

                    let target_create_result = sqlx
                        ::query(
                            "INSERT INTO targets (name, target_date, status, start_value, target_value)
                        VALUES ($1, $2, $3, $4, $5)"
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
                        .execute(&db).await
                        .unwrap();
                    println!("{:?}", target_create_result)
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

    // let result = sqlx::query(
    //     "SELECT name
    //      FROM sqlite_schema
    //      WHERE type ='table'
    //      AND name NOT LIKE 'sqlite_%';",
    // )
    // .fetch_all(&db)
    // .await
    // .unwrap();

    // for (idx, row) in result.iter().enumerate() {
    //     println!("[{}]: {:?}", idx, row.get::<String, &str>("name"));
    // }

    // let result = sqlx::query("INSERT INTO users (name) VALUES (?)")
    //     .bind("bobby")
    //     .execute(&db)
    //     .await
    //     .unwrap();

    // println!("Query result: {:?}", result);

    // let user_results = sqlx::query_as::<_, User>("SELECT id, name, active FROM users")
    //     .fetch_all(&db)
    //     .await
    //     .unwrap();

    // for user in user_results {
    //     println!(
    //         "[{}] name: {}, active: {}",
    //         user.id, &user.name, user.active
    //     );
    // }

    // let delete_result = sqlx::query("DELETE FROM users WHERE name=$1")
    //     .bind("bobby")
    //     .execute(&db)
    //     .await
    //     .unwrap();

    // println!("Delete result: {:?}", delete_result);
}
