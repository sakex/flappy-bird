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
    println!("{}", name);
    let path: PathBuf = format!("./pkg/{}", name).parse().unwrap();
    Ok(NamedFile::open(path)?)
}

#[get("/wasm/snippets/{snippet_name}/{name}")]
async fn serve_wasm_snippet(web::Path((snippet_name, name)): web::Path<(String, String)>) -> Result<NamedFile> {
    let path: PathBuf = format!("./pkg/snippets/{}/{}", snippet_name, name).parse().unwrap();
    Ok(NamedFile::open(path)?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(index)
        .service(serve_wasm)
        .service(serve_wasm_snippet)
        .service(background))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}