
use crate::api_response::ErrorResponse;
use actix_web::{HttpRequest, web, HttpMessage, FromRequest};
use actix_web::http;
use actix_web::error::ErrorUnauthorized;
use actix_web::dev::Payload;
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::future::{ready, Ready};
use uuid::Uuid;

use secrecy::{Secret, ExposeSecret};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}

/*
use actix_web_lab::middleware::Next;
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
*/




#[derive(Debug, Clone)]
pub struct JwtMiddleware(Uuid);

#[derive(Debug, Clone)]
pub struct HmacKey(Secret<String>);


impl FromRequest for JwtMiddleware {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let hmca_key = req.app_data::<web::Data<HmacKey>>().unwrap();

        let token = req
            .headers()
            .get(http::header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .and_then(|h| {
                let words = h.split("Bearer").collect::<Vec<&str>>();
                let token = words.get(1).map(|w| w.trim());

                token
            });
            /*
            .or_else(|| {
                req.cookie("token")
                    .map(|c| c.value())
            });
            */

        if token.is_none() {
            let json_error = ErrorResponse {
                status: "fail".to_string(),
                message: "You are not logged in, please provide token".to_string(),
            };
            return ready(Err(ErrorUnauthorized(json_error)));
        }
        
        let claims = match decode::<TokenClaims>(
            &token.unwrap(),
            &DecodingKey::from_secret(hmca_key.0.expose_secret().as_ref()),
            &Validation::default(),
        ) {
            Ok(c) => c.claims,
            Err(_) => {
                let json_error = ErrorResponse {
                    status: "fail".to_string(),
                    message: "Invalid token".to_string(),
                };
                return ready(Err(ErrorUnauthorized(json_error)));
            }
        };

        let user_id = uuid::Uuid::parse_str(claims.sub.as_str()).unwrap();
        req.extensions_mut()
            .insert::<uuid::Uuid>(user_id.to_owned());

        ready(Ok(JwtMiddleware(user_id)))
    }
}



/*
pub async fn reject_anonymous_user(
    mut req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    
}
*/
