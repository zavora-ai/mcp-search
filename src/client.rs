use anyhow::{Result, bail};
use reqwest::Client;
use serde_json::Value;

#[derive(Clone)]
pub struct ApiClient {
    pub http: Client,
    pub base_url: String,
    pub auth_header: String,
}

impl ApiClient {
    pub async fn get(&self, path: &str) -> Result<Value> {
        let resp = self.http.get(format!("{}{}", self.base_url, path))
            .header("Authorization", &self.auth_header)
            .send().await?;
        if !resp.status().is_success() { bail!("API {}: {}", resp.status(), resp.text().await?); }
        Ok(resp.json().await?)
    }

    pub async fn post(&self, path: &str, body: &Value) -> Result<Value> {
        let resp = self.http.post(format!("{}{}", self.base_url, path))
            .header("Authorization", &self.auth_header)
            .json(body).send().await?;
        if !resp.status().is_success() { bail!("API {}: {}", resp.status(), resp.text().await?); }
        Ok(resp.json().await?)
    }

    pub async fn put(&self, path: &str, body: &Value) -> Result<Value> {
        let resp = self.http.put(format!("{}{}", self.base_url, path))
            .header("Authorization", &self.auth_header)
            .json(body).send().await?;
        if !resp.status().is_success() { bail!("API {}: {}", resp.status(), resp.text().await?); }
        Ok(resp.json().await?)
    }

    pub async fn delete(&self, path: &str) -> Result<Value> {
        let resp = self.http.delete(format!("{}{}", self.base_url, path))
            .header("Authorization", &self.auth_header)
            .send().await?;
        if !resp.status().is_success() { bail!("API {}: {}", resp.status(), resp.text().await?); }
        Ok(resp.json().await?)
    }
}

/// Search backend — supports full-text + vector simultaneously
#[derive(Clone)]
pub struct SearchBackend {
    pub text: ApiClient,
    pub vector: Option<ApiClient>,
}

impl SearchBackend {
    pub fn from_env() -> Result<Self> {
        // Elasticsearch / OpenSearch
        if let Ok(url) = std::env::var("ELASTICSEARCH_URL") {
            let auth = if let Ok(key) = std::env::var("ELASTICSEARCH_API_KEY") {
                format!("ApiKey {}", key)
            } else {
                let user = std::env::var("ELASTICSEARCH_USER").unwrap_or("elastic".into());
                let pass = std::env::var("ELASTICSEARCH_PASSWORD").unwrap_or_default();
                format!("Basic {}", base64::engine::general_purpose::STANDARD.encode(format!("{}:{}", user, pass)))
            };
            tracing::info!("Text search: Elasticsearch");
            let text = ApiClient { http: Client::new(), base_url: url.trim_end_matches('/').into(), auth_header: auth };
            let vector = detect_vector_backend();
            return Ok(Self { text, vector });
        }
        // Algolia
        if let (Ok(app_id), Ok(api_key)) = (std::env::var("ALGOLIA_APP_ID"), std::env::var("ALGOLIA_API_KEY")) {
            tracing::info!("Text search: Algolia");
            let text = ApiClient { http: Client::new(), base_url: format!("https://{}-dsn.algolia.net", app_id), auth_header: format!("Bearer {}", api_key) };
            let vector = detect_vector_backend();
            return Ok(Self { text, vector });
        }
        // Typesense
        if let (Ok(url), Ok(key)) = (std::env::var("TYPESENSE_URL"), std::env::var("TYPESENSE_API_KEY")) {
            tracing::info!("Text search: Typesense");
            let text = ApiClient { http: Client::new(), base_url: url.trim_end_matches('/').into(), auth_header: format!("Bearer {}", key) };
            let vector = detect_vector_backend();
            return Ok(Self { text, vector });
        }
        // Meilisearch
        if let (Ok(url), Ok(key)) = (std::env::var("MEILISEARCH_URL"), std::env::var("MEILISEARCH_API_KEY")) {
            tracing::info!("Text search: Meilisearch");
            let text = ApiClient { http: Client::new(), base_url: url.trim_end_matches('/').into(), auth_header: format!("Bearer {}", key) };
            let vector = detect_vector_backend();
            return Ok(Self { text, vector });
        }
        // Custom API (handles everything)
        if let Ok(url) = std::env::var("SEARCH_API_URL") {
            let key = std::env::var("SEARCH_API_KEY").unwrap_or_default();
            tracing::info!("Search backend: Custom API");
            let text = ApiClient { http: Client::new(), base_url: url.trim_end_matches('/').into(), auth_header: format!("Bearer {}", key) };
            return Ok(Self { text, vector: None });
        }
        bail!("No search backend. Set ELASTICSEARCH_URL, ALGOLIA_APP_ID, TYPESENSE_URL, MEILISEARCH_URL, or SEARCH_API_URL")
    }
}

fn detect_vector_backend() -> Option<ApiClient> {
    // Pinecone
    if let Ok(key) = std::env::var("PINECONE_API_KEY") {
        let host = std::env::var("PINECONE_INDEX_HOST").unwrap_or("https://api.pinecone.io".into());
        tracing::info!("Vector search: Pinecone");
        return Some(ApiClient { http: Client::new(), base_url: host, auth_header: format!("Api-Key {}", key) });
    }
    // Qdrant
    if let Ok(url) = std::env::var("QDRANT_URL") {
        let key = std::env::var("QDRANT_API_KEY").unwrap_or_default();
        tracing::info!("Vector search: Qdrant");
        return Some(ApiClient { http: Client::new(), base_url: url.trim_end_matches('/').into(), auth_header: format!("Bearer {}", key) });
    }
    None
}

use base64::Engine as _;
