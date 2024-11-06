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
use actix_multipart::Multipart;
use std::io;
use std::pin::Pin;
use tokio::sync::mpsc;

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






#[post("/upload")]
async fn upload_file(
    mut payload: Multipart,
    req: HttpRequest,
    storage: web::Data<Arc<Box<dyn Storage>>>,
    db: web::Data<Arc<Box<dyn Database>>>,
) -> impl Responder {
    let uuid = Uuid::new_v4();
    
    if let Ok(Some(mut field)) = payload.try_next().await {
        let filename = match field
            .content_disposition()
            .and_then(|cd| cd.get_filename())
        {
            Some(name) => name.to_string(),
            None => return HttpResponse::BadRequest().body("No filename provided"),
        };
            
        let content_length = match req.headers().get("Content-Length") {
            Some(len) => match len.to_str().ok().and_then(|v| v.parse::<u64>().ok()) {
                Some(len) => len,
                None => return HttpResponse::BadRequest().body("Invalid Content-Length header"),
            },
            None => return HttpResponse::BadRequest().body("Content-Length header required"),
        };

        // Create a channel for streaming the file data
        let (tx, rx) = mpsc::channel::<Result<Bytes, io::Error>>(1024);
        
        // Create a stream from the receiver
        let stream = tokio_stream::wrappers::ReceiverStream::new(rx);
        let pinned_stream = Box::pin(stream);

        // Use the Arc instead of cloning storage
        let storage_clone = Arc::clone(&storage);
        let uuid_string = uuid.to_string();
        
        // Spawn a task to handle the storage write
        let storage_handle = tokio::spawn(async move {
            storage_clone.write_file(&uuid_string, pinned_stream).await
        });

        // Process the field in the current task
        while let Some(chunk) = field.next().await {
            match chunk {
                Ok(data) => {
                    if tx.send(Ok(data)).await.is_err() {
                        break;
                    }
                }
                Err(e) => {
                    let io_error = io::Error::new(
                        io::ErrorKind::Other,
                        format!("Multipart error: {}", e)
                    );
                    let _ = tx.send(Err(io_error)).await;
                    break;
                }
            }
        }
        
        // Drop the sender to signal completion
        drop(tx);

        // Wait for storage to complete
        match storage_handle.await {
            Ok(Ok(())) => {
                let entry = Entry {
                    uuid,
                    file_name: filename,
                    file_size: content_length,
                    source_ip: req.peer_addr()
                        .map(|addr| addr.ip().to_string())
                        .unwrap_or_else(|| "unknown".to_string()),
                    timestamp: Utc::now(),
                };

                match db.create_entry(entry).await {
                    Ok(_) => HttpResponse::Ok().json(uuid),
                    Err(e) => {
                        let _ = storage.delete_file(&uuid.to_string()).await;
                        HttpResponse::InternalServerError()
                            .body(format!("Database error: {}", e))
                    }
                }
            }
            Ok(Err(e)) => HttpResponse::InternalServerError()
                .body(format!("Storage error: {}", e)),
            Err(e) => HttpResponse::InternalServerError()
                .body(format!("Task error: {}", e)),
        }
    } else {
        HttpResponse::BadRequest().body("No file in request")
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
            .service(upload_file)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
