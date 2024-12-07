use std::net::Ipv4Addr;

use actix_web::web;

#[derive(serde::Deserialize)]
struct Info {
    from: Ipv4Addr,
    key: Ipv4Addr,
}

async fn task1(web::Query(Info { from, key }): web::Query<Info>) -> String {
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

pub(crate) fn scope() -> actix_web::Scope {
    web::scope("2").route("/dest", web::get().to(task1))
}
