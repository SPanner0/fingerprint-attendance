use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Sqlite};
use std::env;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Load the environment variables from the .env file
    dotenv::dotenv().expect("unable to load .env file");

    let db_url = env::var("DATABASE_URL").expect("Missing `DATABASE_URL` env var.");

    // Create the database if it doesn't exists
    if !Sqlite::database_exists(&db_url).await.unwrap() {
        println!("Creating databse {:?}", &db_url);
        match Sqlite::create_database(&db_url).await {
            Ok(_) => println!("Successfully created the database"),
            Err(error) => panic!("error: {}", error),
        }
    } else {
        println!("Database already exists");
    }

    // Create the database pool
    let db_pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await
        .expect("Unable to create the database pool");

    // Run the migration files in ./migrations
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Unable to run migrations");
}
