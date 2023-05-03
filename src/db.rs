#![allow(unused_imports)]

use actix_web::Error;
use anyhow::{anyhow, Result as AHResult};
use crate::error::ServiceError;
use deadpool_postgres::{Config, ManagerConfig, RecyclingMethod, Pool as PGPool, Runtime};

/// Config URL is in format:
/// postgres://USER:PASSWORD@HOST:PORT/DBNAME
fn pg_config_from_url(url: &str) -> AHResult<Config> {
    let url = url::Url::parse(url).map_err(|_| anyhow!("could not parse db url"))?;
    let user = url.username().to_string();
    let password = url.password().ok_or(anyhow!("bad password"))?.to_string();
    let host = url.host().ok_or(anyhow!("bad host"))?.to_string();
    // let port = url.port().ok_or("no port in url")?;
    let dbname = url
        .path_segments()
        .ok_or(anyhow!("cannot be base"))?
        .next()
        .unwrap()
        .to_string();

    let mut cfg = Config::new();
    cfg.user = Some(user);
    cfg.password = Some(password);
    cfg.host = Some(host);
    // cfg.port = Some(port);
    cfg.dbname = Some(dbname);

    cfg.manager = Some(ManagerConfig { recycling_method: RecyclingMethod::Fast });

    Ok(cfg)
}

pub fn pg_pool_from_url(url: &str) -> AHResult<PGPool> {
    let pg_cfg = pg_config_from_url(url)?;

    // TODO: Add better error context as one possible error is just 'Bad password'
    Ok(pg_cfg.create_pool(Some(Runtime::Tokio1), tokio_postgres::NoTls)?)
}

// TODO: What to return on Payment Required? An invoice to pay? Generate an invoice?
//       It will likely need the content id and pubkey.
#[allow(unreachable_code)]
#[allow(unused_variables)]
pub async fn check_pubkey_access(pg_pool: PGPool, pubkey: &str, content_id: &str) -> Result<(), Error> {

    return Ok(());

    // UNREACHABLE - PENDING IMPLEMENTATION HERE
    // match pg_pool.get().await {
    //     Ok(pg_conn) => {

    //         // Perform query to check if pubkey has access (i.e. payment or free access granted)
    //         let db_access_check = pg_conn.query_opt("
    //           SELECT
    //             1
    //           FROM content c
    //           LEFT JOIN payments p on p.content_id = c.id
    //           LEFT JOIN identities i on i.id = p.identity_id
    //           WHERE
    //             i.pubkey = $1 AND
    //             p.content_id = $2
    //           LIMIT 1",
    //         &[&pubkey, &content_id]).await;

    //         match db_access_check {
    //             Ok(Some(_db_access_check)) => {
    //                 // This is the success case
    //                 return Ok(())
    //             },
    //             Ok(None) => {
    //                 // TODO: Do we need to set cache avoidance headers here?
    //                 return Err(ServiceError::PaymentRequired.into());
    //             },
    //             Err(_e) => {
    //                 return Err(ServiceError::InternalServerError.into());
    //             }
    //         }
    //     },
    //     Err (_e) => {
    //         return Err(ServiceError::InternalServerError.into());
    //     }
    // }
}
