use crate::database::Database;
use crate::sqlite_database::SqliteDatabase;
use crate::storage::Storage;
use crate::local_storage::LocalStorage;
use actix_web::{web, App, HttpServer, Responder, HttpResponse, get, delete, post, HttpRequest};
use entry::Entry;
use std::env;
use std::sync::Arc;
use uuid::Uuid;
use std::path::PathBuf;
use chrono::Utc;
use futures_util::TryStreamExt;
use tokio_util::io::{StreamReader, ReaderStream};
use bytes::Bytes;
use futures_util::StreamExt;
use actix_web::http::StatusCode;
use futures_util::future::ready;

mod database;
mod entry;
mod sqlite_database;
mod storage;
mod local_storage;

// List all entry UUIDs
#[get("/entries")]
async fn list_entries(db: web::Data<Arc<Box<dyn Database>>>) -> impl Responder {
    match db.list_uuids().await {
        Ok(uuids) => HttpResponse::Ok().json(uuids),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

// Get a specific entry by UUID
#[get("/entry/{uuid}")]
async fn get_entry(
    db: web::Data<Arc<Box<dyn Database>>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let uuid = path.into_inner();
    match db.get_entry(uuid).await {
        Ok(Some(entry)) => HttpResponse::Ok().json(entry),
        Ok(None) => HttpResponse::NotFound().body("Entry not found"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

// Delete an entry by UUID
// This will be updated to also remove the file contents from Storage!
// todo
#[delete("/entry/{uuid}")]
async fn delete_entry(
    db: web::Data<Arc<Box<dyn Database>>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let uuid = path.into_inner();
    match db.delete_entry(uuid).await {
        Ok(true) => HttpResponse::Ok().body("Entry deleted"),
        Ok(false) => HttpResponse::NotFound().body("Entry not found"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}









/// Allows content to be downloaded from the server
#[get("/contents/{uuid}")]
async fn download_file(
    db: web::Data<Arc<Box<dyn Database>>>,
    storage: web::Data<Arc<Box<dyn Storage>>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let uuid = path.into_inner();
    
    // First try to get the entry from database
    match db.get_entry(uuid).await {
        Ok(Some(entry)) => {
            // Entry exists, now try to get the file from storage
            match storage.read_file(&uuid.to_string()).await {
                Ok(stream) => {
                    // Return the file stream with proper headers
                    HttpResponse::Ok()
                        .append_header(("Content-Disposition", format!("attachment; filename=\"{}\"", entry.file_name)))
                        .streaming(stream)
                },
                Err(e) => HttpResponse::InternalServerError().body(format!("Storage error: {}", e)),
            }
        },
        Ok(None) => HttpResponse::NotFound().body("File not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}



#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize database
    let database_path = env::var("DATABASE_PATH").expect("DATABASE_PATH must be set");
    let db: Arc<Box<dyn Database>> = Arc::new(Box::new(
        SqliteDatabase::new(&database_path)
            .await
            .expect("Failed to initialize database"),
    ));
    let db_data = web::Data::new(db);

    // Initialize storage
    let storage_path = env::var("STORAGE_PATH").expect("STORAGE_PATH must be set");
    let storage: Arc<Box<dyn Storage>> = Arc::new(Box::new(
        LocalStorage::new(PathBuf::from(storage_path))
    ));
    let storage_data = web::Data::new(storage);

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(db_data.clone())
            .app_data(storage_data.clone())
            .service(list_entries)
            .service(get_entry)
            .service(delete_entry)
            .service(download_file)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
