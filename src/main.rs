use crate::database::Database;
use crate::sqlite_database::SqliteDatabase;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use std::env;
use std::sync::Arc;
use storage::Storage;

mod database;
mod entry;
mod sqlite_database;
mod storage;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize storage
    let storage = Storage::new();
    println!("Used space: {}", storage.get_used_space().unwrap());

    // Initialize database
    let database_path = env::var("DATABASE_PATH").expect("DATABASE_PATH must be set");
    let db: Arc<Box<dyn Database>> = Arc::new(Box::new(
        SqliteDatabase::new(&database_path)
            .await
            .expect("Failed to initialize database"),
    ));
    let db_data = web::Data::new(db);

    // Start HTTP server
    HttpServer::new(move || App::new().app_data(db_data.clone()))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
