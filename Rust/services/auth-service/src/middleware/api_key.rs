use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse, body::EitherBody,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};

/// API Key middleware for authenticating backend services
/// 
/// Validates X-API-Key header against configured API keys
/// Blocks all requests without valid API key
#[derive(Clone)]
pub struct ApiKeyAuth {
    api_keys: Vec<String>,
}

impl ApiKeyAuth {
    pub fn new(api_keys: Vec<String>) -> Self {
        Self { api_keys }
    }
}

impl<S, B> Transform<S, ServiceRequest> for ApiKeyAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = ApiKeyAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ApiKeyAuthMiddleware {
            service,
            api_keys: self.api_keys.clone(),
        }))
    }
}

pub struct ApiKeyAuthMiddleware<S> {
    service: S,
    api_keys: Vec<String>,
}

impl<S, B> Service<ServiceRequest> for ApiKeyAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let path = req.path();
        
        // Skip API key check for health endpoint
        if path == "/health" {
            let fut = self.service.call(req);
            return Box::pin(async move {
                let response = fut.await?;
                Ok(response.map_into_left_body())
            });
        }

        // Extract API key from header
        let api_key = req
            .headers()
            .get("X-API-Key")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        let is_valid = match api_key {
            Some(key) => self.api_keys.contains(&key),
            None => false,
        };

        if !is_valid {
            tracing::warn!("Unauthorized request to {} - Invalid or missing API key", path);
            
            let response = HttpResponse::Unauthorized()
                .json(serde_json::json!({
                    "error": "Unauthorized",
                    "message": "Valid API key required"
                }));
            
            let fut = self.service.call(req);
            return Box::pin(async move {
                Ok(ServiceResponse::new(
                    fut.await?.request().clone(),
                    response.map_into_right_body()
                ))
            });
        }

        let fut = self.service.call(req);
        Box::pin(async move {
            let response = fut.await?;
            Ok(response.map_into_left_body())
        })
    }
}
