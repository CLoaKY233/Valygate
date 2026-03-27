use async_trait::async_trait;
use reqwest::{Error, Request, Response};

#[async_trait]
pub trait HttpClient: Send + Sync {
    async fn execute(&self, request: Request) -> Result<Response, Error>;
}

#[async_trait]
impl HttpClient for reqwest::Client {
    async fn execute(&self, request: Request) -> Result<Response, Error> {
        self.execute(request).await
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    pub struct MockHttpClient {
        pub requests: Arc<Mutex<Vec<reqwest::Request>>>,
    }

    impl Default for MockHttpClient {
        fn default() -> Self {
            Self::new()
        }
    }

    impl MockHttpClient {
        #[must_use]
        pub fn new() -> Self {
            Self {
                requests: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait]
    impl HttpClient for MockHttpClient {
        async fn execute(
            &self,
            request: reqwest::Request,
        ) -> Result<reqwest::Response, reqwest::Error> {
            self.requests.lock().await.push(request);
            // We can't easily construct a reqwest::Response without making an actual request
            // or using an internal builder. In a real app we'd use `http::Response`
            // and convert, or use `reqwest_mock` crate.
            // This just serves as a skeleton for future testing.
            unimplemented!("MockHttpClient execute is not fully implemented");
        }
    }
}
