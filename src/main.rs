// #[macro_use]
extern crate log;
use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_web_lab::middleware::{from_fn};
use anyhow::Result;
use dotenv::dotenv;
use nostr_paywall_example::AppData;
use nostr_paywall_example::db::pg_pool_from_url;
use nostr_paywall_example::middleware::{authorization_middleware, access_check_middleware};
use nostr_paywall_example::routes::{index, content};


#[actix_web::main]
async fn main() -> Result<()> {

    env_logger::init();

    dotenv().ok();

    let host: String = std::env::var("HOST").expect("HOST must be set.");
    let port: String = std::env::var("PORT").expect("PORT must be set.");

    let auth_event_host: String = std::env::var("AUTH_EVENT_HOST").expect("AUTH_EVENT_HOST must be set.");

    let pg_address: String = std::env::var("POSTGRES_ADDRESS").expect("POSTGRES_ADDRESS must be set.");
    let pg_pool = pg_pool_from_url(&pg_address)?;

    println!("Running Nostr Paywall Server on {host}:{port}");

    HttpServer::new(move || {

        let logger = Logger::default();

        let app_data = AppData{
            pg_pool: pg_pool.clone(),
            auth_event_host: auth_event_host.clone()
        };

        // TODO: Do we need CORs or other middleware here?
        App::new()
            .wrap(logger)
            .app_data(web::Data::new(app_data))
            .service(index)
            .service(
                web::scope("/m/{content_id}")

                // Note: Multiple wrap/wrap_fn call middelware in reversed order
                .wrap(from_fn(access_check_middleware))
                .wrap(from_fn(authorization_middleware))
                .service(content)
        )
    })
    .bind(format!("{host}:{port}"))?
    .run()
    .await?;

    Ok(())
}
