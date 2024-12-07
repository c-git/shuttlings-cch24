use std::net::{Ipv4Addr, Ipv6Addr};

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

#[derive(serde::Deserialize)]
struct Info3Dest {
    from: Ipv6Addr,
    key: Ipv6Addr,
}

async fn task3_dest(web::Query(Info3Dest { from, key }): web::Query<Info3Dest>) -> String {
    let bits = key
        .octets()
        .iter()
        .zip(from.octets())
        .map(|(x, y)| x ^ y)
        .collect::<Vec<u8>>();
    let bits = bits.iter().fold(0u128, |acc, &x| (acc << 8) + x as u128);
    let result = Ipv6Addr::from_bits(bits);
    result.to_string()
}

#[derive(serde::Deserialize)]
struct Info3Key {
    from: Ipv6Addr,
    to: Ipv6Addr,
}

async fn task3_key(web::Query(Info3Key { from, to }): web::Query<Info3Key>) -> String {
    task3_dest(web::Query(Info3Dest { from, key: to })).await
}

pub(crate) fn scope() -> actix_web::Scope {
    web::scope("/2")
        .route("/dest", web::get().to(task1))
        .route("/key", web::get().to(task2))
        .service(
            web::scope("/v6")
                .route("/dest", web::get().to(task3_dest))
                .route("/key", web::get().to(task3_key)),
        )
}
