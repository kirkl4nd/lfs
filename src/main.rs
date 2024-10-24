use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use storage::Storage;
use uuid::Uuid;

mod entry;
mod database;
mod sqlite_database;
mod storage;

#[get("/hello-world")]
async fn hello_world() -> impl Responder {
    HttpResponse::Ok().body("Hello World!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    dotenv::dotenv().ok();

    let storage = Storage::new();
    println!("Used space: {}", storage.get_used_space().unwrap());
    HttpServer::new(|| {
        App::new()
            .service(hello_world)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}


#[get("/api/download/{uuid}")]
async fn get_file(path: web::Path<Uuid>) -> impl Responder {
    let uuid = path.into_inner();
    
    // check if uuid is in the database

    // get the file contents from storage and send them to the client such that the file will be downloaded
    // the files in storage are named by uuid. we must change the file name in the response to the filename in the database entry
    // the file in storage should NOT be modified or renamed


    todo!()
}