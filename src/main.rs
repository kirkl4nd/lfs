use crate::database::Database;
use crate::sqlite_database::SqliteDatabase;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use std::env;
use storage::Storage;
use uuid::Uuid;

mod database;
mod entry;
mod sqlite_database;
mod storage;

#[get("/hello-world")]
async fn hello_world() -> impl Responder {
    HttpResponse::Ok().body("Hello World!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize database
    let database_path = env::var("DATABASE_PATH").expect("DATABASE_PATH must be set");
    let db = SqliteDatabase::new(&database_path)
        .await
        .expect("Failed to initialize database");
    let db_data = web::Data::new(db);

    // Start HTTP server
    HttpServer::new(move || App::new().app_data(db_data.clone()).service(hello_world))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
