use actix_web::{web, HttpResponse};

pub async fn task1() -> &'static str {
    "Hello, bird!"
}

async fn task2() -> HttpResponse {
    HttpResponse::Found()
        .insert_header(("Location", "https://www.youtube.com/watch?v=9Gc4QTqslN4"))
        .finish()
}

pub(crate) fn scope() -> actix_web::Scope {
    web::scope("-1").route("/seek", web::get().to(task2))
}
