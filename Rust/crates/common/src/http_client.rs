// HTTP client wrapper for inter-service communication
#[allow(dead_code)]
pub struct HttpClient {
    client: reqwest::Client,
}

impl HttpClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}
