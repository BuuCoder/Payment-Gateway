use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};

use crate::jwt::JwtValidator;

pub struct AuthMiddleware {
    validator: JwtValidator,
}

impl AuthMiddleware {
    pub fn new(jwt_secret: String) -> Self {
        Self {
            validator: JwtValidator::new(jwt_secret),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service,
            validator: self.validator.clone(),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: S,
    validator: JwtValidator,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Extract Authorization header
        let auth_header = req.headers().get("Authorization");

        let token = match auth_header {
            Some(header) => {
                let header_str = match header.to_str() {
                    Ok(s) => s,
                    Err(_) => {
                        return Box::pin(async {
                            Err(actix_web::error::ErrorUnauthorized(
                                serde_json::json!({"error": "Invalid authorization header"}),
                            ))
                        });
                    }
                };

                // Extract token from "Bearer <token>"
                if let Some(token) = header_str.strip_prefix("Bearer ") {
                    token.to_string()
                } else {
                    return Box::pin(async {
                        Err(actix_web::error::ErrorUnauthorized(
                            serde_json::json!({"error": "Invalid authorization format"}),
                        ))
                    });
                }
            }
            None => {
                return Box::pin(async {
                    Err(actix_web::error::ErrorUnauthorized(
                        serde_json::json!({"error": "Missing authorization header"}),
                    ))
                });
            }
        };

        // Verify token
        let claims = match self.validator.verify_token(&token) {
            Ok(claims) => claims,
            Err(e) => {
                return Box::pin(async move {
                    Err(actix_web::error::ErrorUnauthorized(
                        serde_json::json!({"error": format!("Unauthorized: {}", e)}),
                    ))
                });
            }
        };

        // Insert claims into request extensions
        req.extensions_mut().insert(claims.clone());

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}
