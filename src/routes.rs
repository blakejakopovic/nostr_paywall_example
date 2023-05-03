#![allow(unused_variables)]
#![allow(unused_imports)]

use crate::{Pubkey, AppData};
use actix_web::{get, web::{self, Data}, Responder, HttpResponse};

// Example homepage without any auth required
#[get("/")]
async fn index() -> impl Responder {
    "Hello, Nostr!"
}

// Example content requiring an auth check
// Note: The 'get' path is blank, as it's set as part of a web::scope
#[get("")]
pub async fn content(content_id: web::Path<String>, pubkey: web::ReqData<Pubkey>, app_data: Data<AppData>) -> impl Responder {

    let pubkey = pubkey.into_inner().pubkey;
    debug!("pubkey: {pubkey:?}");

    format!("Hello, {pubkey}! Paid Content: {content_id}")

    // EXAMPLE: Lookup content based on content_id and return content
    // let pg_conn = app_data.pg_pool.get().await.expect("db connection");

    // let pg_content = pg_conn.query_opt("
    //     SELECT content_type, content_data
    //     FROM content
    //     WHERE
    //       content_id = $1
    //     LIMIT 1;
    // ", &[&content_id.into_inner()]).await;

    // match pg_content {
    //     Ok(Some(pg_content)) => {

    //         // TODO:
    //         //     if URL, it can 302 redirect to content
    //         //     if File, it can serve file with Mime Type
    //         //     It can alternatively leverage an existing file server and proxy to that directly

    //         HttpResponse::Ok().finish()
    //     },
    //     Ok(None) => {
    //         // TODO: Do we need to set don't cache headers here?
    //         HttpResponse::NotFound().finish()
    //     },
    //     Err(_e) => {
    //         HttpResponse::InternalServerError().finish()
    //     }
    // }
}
