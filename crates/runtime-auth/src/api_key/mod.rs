/*
Copyright 2024 The Spice.ai OSS Authors

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

     https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

use axum::http;

use crate::{error::Error, AuthVerdict, HttpAuth};

pub struct ApiKeyAuth {
    api_keys: Vec<String>,
}

impl ApiKeyAuth {
    #[must_use]
    pub fn new(api_keys: Vec<String>) -> Self {
        Self { api_keys }
    }
}

impl HttpAuth for ApiKeyAuth {
    /// Checks the `X-API-Key` header for a valid API key
    fn http(&self, request: &http::request::Parts) -> Result<AuthVerdict, Error> {
        let api_key = request
            .headers
            .get("X-API-Key")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default();

        if self.api_keys.iter().any(|key| key == api_key) {
            Ok(AuthVerdict::Allow)
        } else {
            Ok(AuthVerdict::Deny)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::request::Builder;

    fn create_request_parts(api_key: Option<&str>) -> http::request::Parts {
        let mut builder = Builder::new().uri("https://example.com");

        if let Some(key) = api_key {
            builder = builder.header("X-API-Key", key);
        }

        let request = builder.body(()).expect("Failed to build request");
        request.into_parts().0
    }

    #[test]
    fn test_valid_api_key() {
        let auth = ApiKeyAuth::new(vec!["valid-key".to_string()]);
        let parts = create_request_parts(Some("valid-key"));

        let result = auth.http(&parts);
        assert!(matches!(result, Ok(AuthVerdict::Allow)));
    }

    #[test]
    fn test_invalid_api_key() {
        let auth = ApiKeyAuth::new(vec!["valid-key".to_string()]);
        let parts = create_request_parts(Some("invalid-key"));

        let result = auth.http(&parts);
        assert!(matches!(result, Ok(AuthVerdict::Deny)));
    }

    #[test]
    fn test_missing_api_key() {
        let auth = ApiKeyAuth::new(vec!["valid-key".to_string()]);
        let parts = create_request_parts(None);

        let result = auth.http(&parts);
        assert!(matches!(result, Ok(AuthVerdict::Deny)));
    }

    #[test]
    fn test_multiple_valid_keys() {
        let auth = ApiKeyAuth::new(vec![
            "key1".to_string(),
            "key2".to_string(),
            "key3".to_string(),
        ]);

        let parts = create_request_parts(Some("key2"));
        let result = auth.http(&parts);
        assert!(matches!(result, Ok(AuthVerdict::Allow)));
    }
}