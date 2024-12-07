use std::net::Ipv4Addr;

use actix_web::web;

#[derive(serde::Deserialize)]
struct Info1 {
    from: Ipv4Addr,
    key: Ipv4Addr,
}

async fn task1(web::Query(Info1 { from, key }): web::Query<Info1>) -> String {
    let [a, b, c, d] = from
        .octets()
        .iter()
        .zip(key.octets())
        .map(|(x, y)| x.overflowing_add(y).0)
        .collect::<Vec<u8>>()[..]
    else {
        unreachable!("Always has 4 octets")
    };
    let result = Ipv4Addr::new(a, b, c, d);
    result.to_string()
}

#[derive(serde::Deserialize)]
struct Info2 {
    from: Ipv4Addr,
    to: Ipv4Addr,
}

async fn task2(web::Query(Info2 { from, to }): web::Query<Info2>) -> String {
    let [a, b, c, d] = to
        .octets()
        .iter()
        .zip(from.octets())
        .map(|(x, y)| x.overflowing_sub(y).0)
        .collect::<Vec<u8>>()[..]
    else {
        unreachable!("Always has 4 octets")
    };
    let result = Ipv4Addr::new(a, b, c, d);
    result.to_string()
}

pub(crate) fn scope() -> actix_web::Scope {
    web::scope("2")
        .route("/dest", web::get().to(task1))
        .route("/key", web::get().to(task2))
}
