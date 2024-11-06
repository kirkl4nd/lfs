use crate::database::Database;
use crate::sqlite_database::SqliteDatabase;
use crate::storage::Storage;
use crate::local_storage::LocalStorage;
use actix_web::{web, App, HttpServer, Responder, HttpResponse, get, delete, post, HttpRequest};
use entry::Entry;
use storage::{DeleteFileResult, ReadFileResult, WriteFileResult};
use std::env;
use std::sync::Arc;
use uuid::Uuid;
use std::path::PathBuf;
use chrono::Utc;
use futures_util::TryStreamExt;
use bytes::Bytes;
use futures_util::StreamExt;
use actix_multipart::Multipart;
use std::io;
use tokio::sync::mpsc;
use actix_web::middleware::DefaultHeaders;
use actix_files::NamedFile;
use std::path::Path;
use actix_cors::Cors;

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

// Delete an entry by UUID and its associated file from storage
#[delete("/entry/{uuid}")]
async fn delete_entry(
    db: web::Data<Arc<Box<dyn Database>>>,
    storage: web::Data<Arc<Box<dyn Storage>>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let uuid = path.into_inner();
    
    // First check if entry exists in database
    let entry = match db.get_entry(uuid).await {
        Ok(Some(entry)) => entry,
        Ok(None) => return HttpResponse::NotFound().body("Entry not found"),
        Err(e) => return HttpResponse::InternalServerError()
            .body(format!("Database error: {}", e)),
    };

    // Delete from database
    match db.delete_entry(uuid).await {
        Ok(true) => {
            // Try to delete from storage
            match storage.delete_file(&uuid.to_string()).await {
                DeleteFileResult::Success => {
                    HttpResponse::Ok().body("Entry deleted")
                },
                DeleteFileResult::NotFound => {
                    // File not found in storage but database entry was deleted - this is ok
                    HttpResponse::Ok().body("Entry deleted")
                },
                DeleteFileResult::Failure(e) => {
                    // Storage deletion failed - attempt to restore database entry
                    if let Err(db_err) = db.create_entry(entry).await {
                        return HttpResponse::InternalServerError()
                            .body(format!("Critical error: Storage deletion failed AND database restoration failed. Storage error: {}, Database error: {}", e, db_err));
                    }
                    HttpResponse::InternalServerError()
                        .body(format!("Storage error: {}", e))
                }
            }
        },
        Ok(false) => HttpResponse::InternalServerError()
            .body("Database entry not removed"),
        Err(e) => HttpResponse::InternalServerError()
            .body(format!("Database error: {}", e)),
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
    
    match db.get_entry(uuid).await {
        Ok(Some(entry)) => {
            match storage.read_file(&uuid.to_string()).await {
                ReadFileResult::Success(stream) => {
                    HttpResponse::Ok()
                        .append_header(("Content-Type", "application/octet-stream"))
                        .append_header(("Content-Disposition", format!("attachment; filename=\"{}\"", entry.file_name)))
                        .streaming(stream)
                },
                ReadFileResult::NotFound => {
                    HttpResponse::NotFound()
                        .body("File contents are missing from storage")
                },
                ReadFileResult::Failure(e) => {
                    HttpResponse::InternalServerError()
                        .body(format!("Storage error: {}", e))
                }
            }
        },
        Ok(None) => HttpResponse::NotFound().body("Entry not found in database"),
        Err(e) => HttpResponse::InternalServerError()
            .body(format!("Database error: {}", e)),
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
                        return HttpResponse::InternalServerError()
                            .body("Failed to process upload stream");
                    }
                }
                Err(e) => {
                    let io_error = io::Error::new(
                        io::ErrorKind::Other,
                        format!("Multipart error: {}", e)
                    );
                    let _ = tx.send(Err(io_error)).await;
                    return HttpResponse::InternalServerError()
                        .body(format!("Upload error: {}", e));
                }
            }
        }
        
        // Drop the sender to signal completion
        drop(tx);

        // Wait for storage to complete and handle the result
        match storage_handle.await {
            Ok(WriteFileResult::Success) => {
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
                        // Clean up the stored file if database entry fails
                        match storage.delete_file(&uuid.to_string()).await {
                            DeleteFileResult::Success | DeleteFileResult::NotFound => (),
                            DeleteFileResult::Failure(cleanup_err) => {
                                eprintln!("Failed to clean up file after database error: {}", cleanup_err);
                            }
                        }
                        HttpResponse::InternalServerError()
                            .body(format!("Database error: {}", e))
                    }
                }
            }
            Ok(WriteFileResult::Failure(e)) => {
                HttpResponse::InternalServerError()
                    .body(format!("Storage error: {}", e))
            }
            Err(e) => HttpResponse::InternalServerError()
                .body(format!("Task error: {}", e))
        }
    } else {
        HttpResponse::BadRequest().body("No file in request")
    }
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(include_str!("web/test.html"))
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
            .wrap(Cors::default()
                .allow_any_origin()
                .allow_any_method()
                .allow_any_header()
                .max_age(3600))
            .app_data(db_data.clone())
            .app_data(storage_data.clone())
            .service(index)
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
