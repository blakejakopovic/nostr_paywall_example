use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::HttpMessage;
use actix_web::{Error, web::Data};
use actix_web_lab::middleware::Next;
use anyhow::{anyhow, Result as AHResult};
use base64::{Engine as _, engine::general_purpose};
use crate::db::check_pubkey_access;
use crate::error::ServiceError;
use crate::{AUTH_EVENT_KIND,AUTH_EVENT_CREATED_AT_DELTA_SEC, AppData, Pubkey};
use nostr_rs_relay::event::Event;
use std::time::{SystemTime, UNIX_EPOCH};

fn validate_absolute_path_match(event: &Event, req: &ServiceRequest) -> bool {
    let app_data: &Data<AppData> = req.app_data().expect("app data");
    let auth_event_host = &app_data.auth_event_host;

    let abs_path = format!("{auth_event_host}{path}", path = req.path());
    event.tags.iter().any(|tag| tag == &["u", &abs_path])
}

fn validate_method_match(event: &Event, req: &ServiceRequest) -> bool {
    let method = req.method().as_str();

    // Note: Method must be uppercase
    event.tags.iter().any(|tag| tag == &["method", method])
}

fn validate_auth_event<'a>(event: &'a Event, req: &'a ServiceRequest) -> AHResult<&'a Event> {

    // Check the event signature, pubkey and id
    if event.validate().is_err() {
      warn!("Event signature is invalid: {event:?}");
      return Err(anyhow!("Event signature is invalid: {event:?}"))
    }

    // Check the event created_at is not stale (within past N seconds)
    let now = SystemTime::now()
                  .duration_since(UNIX_EPOCH)?
                  .as_secs();

    // We allow a future time drift buffer, and a max created_at lifetime
    // TODO: Need to validate this time bound actually works..
    if !((now - AUTH_EVENT_CREATED_AT_DELTA_SEC)..=(now + AUTH_EVENT_CREATED_AT_DELTA_SEC)).contains(&event.created_at) {

      warn!("Invalid event created_at: {}", event.created_at);
      return Err(anyhow!("Invalid event created_at: {}", event.created_at))
    }

    // Check the event kind is correct
    if event.kind != AUTH_EVENT_KIND {
      warn!("Invalid event kind: {}", event.kind);
      return Err(anyhow!("Invalid event kind: {}", event.kind))
    }

    if !validate_absolute_path_match(&event, &req) {
        warn!("Invalid event u tag");
        return Err(anyhow!("Invalid event u tag value"))
    }

    if !validate_method_match(&event, &req) {
        warn!("Invalid event method tag");
        return Err(anyhow!("Invalid event method tag value"))
    }

    return Ok(event);
}

// NIP 98 - HTTP Auth
// https://github.com/nostr-protocol/nips/blob/af4cbfbddb2900b7bc4a56b57430989e8b613006/98.md
// {
//   "id": "cddeb723c58a2ff78cd5f0f315d855306ce7eada1714a26572ac0f0d2e619925",
//   "pubkey": "970e0ebe27a552be8982047974f60302caab0fcb9aa86855f81102e67e084e45",
//   "created_at": 1682772206,
//   "kind": 27235,
//   "tags": [
//     [
//       "u",
//       "http://localhost:8080/m/test_content_id"
//     ],
//     [
//       "method",
//       "GET"
//     ]
//   ],
//   "content": "",
//   "sig": "2ef81b6b7d494d57dc06ac261aeb1e9c7dfeeb3868b2c4daee604ccdf4cb9cda8f3df08c50f93c99e327ad8fb40175907d47f7dfc100633af3fe38f085d6c888"
// }
fn verify_auth_header<'a>(header_value: &'a str, req: &'a ServiceRequest) -> Result<String, Error> {

    // Check and strip 'nostr<space>' header value prefix
    let encoded_auth = match header_value {
        s if s.to_lowercase().starts_with("nostr ") => {
            s[6..].to_owned()
        },
        _ => {
            return Err(ServiceError::BadRequest.into())
        }
    };
    debug!("encoded_auth: {encoded_auth}");

    // Base64 decode
    let decoded_auth = match general_purpose::STANDARD.decode(encoded_auth) {
        Ok(decoded_auth) => decoded_auth,
        Err(_e) => return Err(ServiceError::BadRequest.into())
    };

    // Decode UTF8
    let auth_event_json = match std::str::from_utf8(&decoded_auth) {
        Ok(auth_event_json) => auth_event_json,
        Err(_e) => return Err(ServiceError::BadRequest.into())
    };
    debug!("auth_event_json: {auth_event_json:?}");

    // Deserialize Event json
    let event: Event = match serde_json::from_str(auth_event_json) {
        Ok(event) => event,
        Err(_e) => return Err(ServiceError::BadRequest.into())
    };
    debug!("event: {event:?}");

    // Validate event for request
    let pubkey: String = match validate_auth_event(&event, &req) {
        Ok(event) => event.pubkey.to_owned(),
        Err(_e) => return Err(ServiceError::BadRequest.into())
    };
    debug!("pubkey: {pubkey:?}");

    Ok(pubkey)
}

pub async fn authorization_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {

    let auth_header_value = req.headers()
      .get("AUTHORIZATION")
      .map(|value| value.to_str().ok())
      .ok_or(ServiceError::PaymentRequired)?;

    // TODO: Store in request extension
    let pubkey: String = match auth_header_value {
        Some(t) => {
            verify_auth_header(&t, &req)?
        },
        None => {
            return Err(ServiceError::BadRequest.into())
        }
    };

    // Append pubkey to the request data available for route handlers
    let pubkey_ext = Pubkey { pubkey: pubkey.clone() };
    req.extensions_mut().insert(pubkey_ext);

    debug!("authorization_middleware: complete");

    // Call next middleware
    next.call(req).await
}

pub async fn access_check_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {

    let app_data: &Data<AppData> = req.app_data().expect("app data");
    let pg_pool = &app_data.pg_pool;

    // Load prior extracted and validated pubkey from authorization_middleware
    let pubkey = match req.extensions().get::<Pubkey>() {
        Some(pubkey_ext) => {
            pubkey_ext.clone().into_inner()
        },
        None => return Err(ServiceError::InternalServerError.into()),
    };

    // Check access
    let content_id = req.match_info().query("content_id");
    check_pubkey_access(pg_pool.clone(), &pubkey, content_id).await?;

    debug!("access_check_middleware: complete");

    // Call next middleware
    next.call(req).await
}
