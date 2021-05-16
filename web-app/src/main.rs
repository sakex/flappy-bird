use actix_files::NamedFile;
use actix_web::{get, web, App, HttpServer, Result};
use std::path::PathBuf;

#[get("/")]
async fn index() -> Result<NamedFile> {
    let path: PathBuf = "./files/index.html".parse().unwrap();
    Ok(NamedFile::open(path)?)
}

#[get("/background")]
async fn background() -> Result<NamedFile> {
    let path: PathBuf = "./files/background.jpeg".parse().unwrap();
    Ok(NamedFile::open(path)?)
}

#[get("/wasm/{name}")]
async fn serve_wasm(web::Path(name): web::Path<String>) -> Result<NamedFile> {
    let path: PathBuf = format!("./pkg/{}", name).parse().unwrap();
    Ok(NamedFile::open(path)?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(index)
        .service(serve_wasm)
        .service(background))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}