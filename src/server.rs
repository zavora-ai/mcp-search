use crate::client::SearchBackend;
use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use serde_json::json;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct EmptyInput {}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NameInput { pub name: String }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchInput { pub query: String, pub index: String, pub filters: Option<serde_json::Value>, pub facets: Option<Vec<String>>, pub limit: Option<u32>, pub offset: Option<u32> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SemanticInput { pub vector: Vec<f64>, pub index: Option<String>, pub namespace: Option<String>, pub top_k: Option<u32>, pub filters: Option<serde_json::Value> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MultiSearchInput { pub queries: Vec<SearchQuery> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchQuery { pub query: String, pub index: String, pub limit: Option<u32> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SuggestInput { pub query: String, pub index: String, pub limit: Option<u32> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SimilarInput { pub document_id: String, pub index: String, pub top_k: Option<u32> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CreateIndexInput { pub name: String, pub fields: serde_json::Value }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct IndexDocInput { pub index: String, pub id: Option<String>, pub document: serde_json::Value }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DeleteDocInput { pub index: String, pub id: String }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct UpsertVectorsInput { pub namespace: Option<String>, pub vectors: Vec<VectorEntry> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct VectorEntry { pub id: String, pub values: Vec<f64>, pub metadata: Option<serde_json::Value> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct QueryVectorsInput { pub vector: Vec<f64>, pub namespace: Option<String>, pub top_k: Option<u32>, pub filters: Option<serde_json::Value> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ExplainInput { pub query: String, pub document_id: String, pub index: String }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SynonymsInput { pub index: String, pub rules: Option<Vec<SynonymRule>> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SynonymRule { pub input: String, pub synonyms: Vec<String> }

#[derive(Clone)]
pub struct SearchServer { pub backend: SearchBackend }

fn r(result: Result<serde_json::Value, anyhow::Error>) -> String {
    match result { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {}", e) }
}

#[tool_router(server_handler)]
impl SearchServer {
    // === Search (5) ===

    #[tool(description = "Full-text search with query, filters, facets, and pagination")]
    async fn search(&self, Parameters(input): Parameters<SearchInput>) -> String {
        r(self.backend.text.post("/search", &json!({
            "query": input.query, "index": input.index, "filters": input.filters,
            "facets": input.facets, "limit": input.limit.unwrap_or(10), "offset": input.offset.unwrap_or(0)
        })).await)
    }

    #[tool(description = "Vector/semantic similarity search using embeddings")]
    async fn semantic_search(&self, Parameters(input): Parameters<SemanticInput>) -> String {
        let api = self.backend.vector.as_ref().unwrap_or(&self.backend.text);
        r(api.post("/search/semantic", &json!({
            "vector": input.vector, "index": input.index, "namespace": input.namespace,
            "top_k": input.top_k.unwrap_or(10), "filters": input.filters
        })).await)
    }

    #[tool(description = "Search across multiple indexes simultaneously")]
    async fn multi_search(&self, Parameters(input): Parameters<MultiSearchInput>) -> String {
        let queries: Vec<serde_json::Value> = input.queries.iter().map(|q| json!({"query": q.query, "index": q.index, "limit": q.limit.unwrap_or(5)})).collect();
        r(self.backend.text.post("/search/multi", &json!({"queries": queries})).await)
    }

    #[tool(description = "Get autocomplete/typeahead suggestions")]
    async fn suggest(&self, Parameters(input): Parameters<SuggestInput>) -> String {
        r(self.backend.text.get(&format!("/suggest?q={}&index={}&limit={}", urlencoding::encode(&input.query), input.index, input.limit.unwrap_or(5))).await)
    }

    #[tool(description = "Find documents similar to a given document")]
    async fn find_similar(&self, Parameters(input): Parameters<SimilarInput>) -> String {
        r(self.backend.text.post("/similar", &json!({"document_id": input.document_id, "index": input.index, "top_k": input.top_k.unwrap_or(5)})).await)
    }

    // === Index Management (5) ===

    #[tool(description = "List all search indexes/collections")]
    async fn list_indexes(&self, Parameters(_): Parameters<EmptyInput>) -> String {
        r(self.backend.text.get("/indexes").await)
    }

    #[tool(description = "Get index details: schema, document count, size")]
    async fn get_index(&self, Parameters(input): Parameters<NameInput>) -> String {
        r(self.backend.text.get(&format!("/indexes/{}", input.name)).await)
    }

    #[tool(description = "Create a new search index with field mappings")]
    async fn create_index(&self, Parameters(input): Parameters<CreateIndexInput>) -> String {
        r(self.backend.text.post("/indexes", &json!({"name": input.name, "fields": input.fields})).await)
    }

    #[tool(description = "Add or update a document in an index")]
    async fn index_document(&self, Parameters(input): Parameters<IndexDocInput>) -> String {
        r(self.backend.text.post(&format!("/indexes/{}/documents", input.index), &json!({"id": input.id, "document": input.document})).await)
    }

    #[tool(description = "Remove a document from an index")]
    async fn delete_document(&self, Parameters(input): Parameters<DeleteDocInput>) -> String {
        r(self.backend.text.delete(&format!("/indexes/{}/documents/{}", input.index, input.id)).await)
    }

    // === Vector Operations (4) ===

    #[tool(description = "Insert or update vector embeddings")]
    async fn upsert_vectors(&self, Parameters(input): Parameters<UpsertVectorsInput>) -> String {
        let api = self.backend.vector.as_ref().unwrap_or(&self.backend.text);
        let vectors: Vec<serde_json::Value> = input.vectors.iter().map(|v| json!({"id": v.id, "values": v.values, "metadata": v.metadata})).collect();
        r(api.post("/vectors/upsert", &json!({"namespace": input.namespace, "vectors": vectors})).await)
    }

    #[tool(description = "Query vectors by similarity (top-k nearest neighbors)")]
    async fn query_vectors(&self, Parameters(input): Parameters<QueryVectorsInput>) -> String {
        let api = self.backend.vector.as_ref().unwrap_or(&self.backend.text);
        r(api.post("/vectors/query", &json!({
            "vector": input.vector, "namespace": input.namespace,
            "top_k": input.top_k.unwrap_or(10), "filters": input.filters
        })).await)
    }

    #[tool(description = "List vector namespaces/collections")]
    async fn list_namespaces(&self, Parameters(_): Parameters<EmptyInput>) -> String {
        let api = self.backend.vector.as_ref().unwrap_or(&self.backend.text);
        r(api.get("/vectors/namespaces").await)
    }

    #[tool(description = "Get vector index stats: count, dimensions, fullness")]
    async fn get_vector_stats(&self, Parameters(_): Parameters<EmptyInput>) -> String {
        let api = self.backend.vector.as_ref().unwrap_or(&self.backend.text);
        r(api.get("/vectors/stats").await)
    }

    // === Analytics & Optimization (4) ===

    #[tool(description = "Get search analytics: top queries, CTR, zero-result queries")]
    async fn get_search_analytics(&self, Parameters(_): Parameters<EmptyInput>) -> String {
        r(self.backend.text.get("/analytics/queries").await)
    }

    #[tool(description = "Get index stats: document count, size, field cardinality")]
    async fn get_index_stats(&self, Parameters(input): Parameters<NameInput>) -> String {
        r(self.backend.text.get(&format!("/indexes/{}/stats", input.name)).await)
    }

    #[tool(description = "Explain why a document ranked where it did for a query")]
    async fn explain_ranking(&self, Parameters(input): Parameters<ExplainInput>) -> String {
        r(self.backend.text.post("/search/explain", &json!({"query": input.query, "document_id": input.document_id, "index": input.index})).await)
    }

    #[tool(description = "Test a query and see scoring breakdown")]
    async fn test_query(&self, Parameters(input): Parameters<SearchInput>) -> String {
        r(self.backend.text.post("/search/test", &json!({"query": input.query, "index": input.index, "limit": input.limit.unwrap_or(5)})).await)
    }

    // === Configuration (2) ===

    #[tool(description = "Get configured synonym rules for an index")]
    async fn get_synonyms(&self, Parameters(input): Parameters<NameInput>) -> String {
        r(self.backend.text.get(&format!("/synonyms?index={}", input.name)).await)
    }

    #[tool(description = "Add or update synonym rules")]
    async fn update_synonyms(&self, Parameters(input): Parameters<SynonymsInput>) -> String {
        let rules: Vec<serde_json::Value> = input.rules.unwrap_or_default().iter().map(|r| json!({"input": r.input, "synonyms": r.synonyms})).collect();
        r(self.backend.text.put("/synonyms", &json!({"index": input.index, "rules": rules})).await)
    }
}
