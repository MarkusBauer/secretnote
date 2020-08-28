use actix_web::{get, web, App, HttpServer, Responder, middleware};
use actix_files;
use std::env;

#[get("/")]
async fn front() -> impl Responder {
    format!("Hello World!")
}

#[get("/{id}/{name}/index.html")]
async fn index(web::Path((id, name)): web::Path<(u32, String)>) -> impl Responder {
    format!("Hello {}! id:{}", name, id)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .service(front)
            .service(index)
            .service(actix_files::Files::new("/static", "/home/markus/Projekte/secretnote/static").show_files_listing())
    }).bind("127.0.0.1:8080")?.run().await
}