mod common;

use crate::common::*;

async fn index() -> impl Responder {
    let html = std::fs::read_to_string("template/main.html").unwrap();
    HttpResponse::Ok().content_type("text/html").body(html)
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))  // HTML 파일 제공
            .service(Files::new("/static", "static").show_files_listing()) // 정적 파일 제공 (CSS, JS)
    })
    .bind("127.0.0.1:8999")?
    .run()
    .await
}