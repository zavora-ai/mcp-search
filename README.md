# Search MCP Server

[![Crates.io](https://img.shields.io/crates/v/mcp-search.svg)](https://crates.io/crates/mcp-search)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![ADK-Rust Enterprise](https://img.shields.io/badge/ADK--Rust-Enterprise-purple.svg)](https://enterprise.adk-rust.com)

Unified search for AI agents — full-text, vector/semantic, and structured search with index management, analytics, and synonym configuration. 20 tools supporting 7 backends with dual-mode (text + vector simultaneously).

## Architecture

<p align="center">
  <img src="https://raw.githubusercontent.com/zavora-ai/mcp-search/main/docs/assets/architecture.svg" alt="MCP Search Architecture" width="850"/>
</p>

## Tools (20)

### Search (5)

| Tool | Purpose | Risk |
|------|---------|------|
| `search` | Full-text search with filters, facets, pagination | read_only |
| `semantic_search` | Vector similarity search using embeddings | read_only |
| `multi_search` | Search across multiple indexes at once | read_only |
| `suggest` | Autocomplete/typeahead suggestions | read_only |
| `find_similar` | Find documents similar to a given one | read_only |

### Index Management (5)

| Tool | Purpose | Risk |
|------|---------|------|
| `list_indexes` | List all indexes/collections | read_only |
| `get_index` | Index schema, doc count, size | read_only |
| `create_index` | Create index with field mappings | internal_write |
| `index_document` | Add/update a document | internal_write |
| `delete_document` | Remove a document | internal_write |

### Vector Operations (4)

| Tool | Purpose | Risk |
|------|---------|------|
| `upsert_vectors` | Insert/update embeddings | internal_write |
| `query_vectors` | Top-k nearest neighbors | read_only |
| `list_namespaces` | List vector namespaces | read_only |
| `get_vector_stats` | Count, dimensions, fullness | read_only |

### Analytics & Optimization (4)

| Tool | Purpose | Risk |
|------|---------|------|
| `get_search_analytics` | Top queries, CTR, zero-results | read_only |
| `get_index_stats` | Doc count, size, cardinality | read_only |
| `explain_ranking` | Why a doc ranked where it did | read_only |
| `test_query` | Test query with scoring breakdown | read_only |

### Configuration (2)

| Tool | Purpose | Risk |
|------|---------|------|
| `get_synonyms` | List synonym rules | read_only |
| `update_synonyms` | Add/update synonyms | internal_write |

## Installation

```bash
cargo install mcp-search
```

## Configuration

### Full-Text Backends (pick one)

| Backend | Env Vars |
|---------|----------|
| **Elasticsearch** | `ELASTICSEARCH_URL` + `ELASTICSEARCH_API_KEY` (or `_USER`+`_PASSWORD`) |
| **Algolia** | `ALGOLIA_APP_ID` + `ALGOLIA_API_KEY` |
| **Typesense** | `TYPESENSE_URL` + `TYPESENSE_API_KEY` |
| **Meilisearch** | `MEILISEARCH_URL` + `MEILISEARCH_API_KEY` |

### Vector Backends (optional, additive)

| Backend | Env Vars |
|---------|----------|
| **Pinecone** | `PINECONE_API_KEY` + `PINECONE_INDEX_HOST` |
| **Qdrant** | `QDRANT_URL` + `QDRANT_API_KEY` |

### Universal

| Backend | Env Vars |
|---------|----------|
| **Custom API** | `SEARCH_API_URL` + `SEARCH_API_KEY` |

### Dual-Mode Example

```bash
# Elasticsearch for full-text + Pinecone for vectors
export ELASTICSEARCH_URL="https://my-cluster.es.cloud:9243"
export ELASTICSEARCH_API_KEY="base64key"
export PINECONE_API_KEY="pc-xxxxx"
export PINECONE_INDEX_HOST="https://my-index-xxxxx.svc.pinecone.io"
mcp-search
```

## Client Configuration

```json
{
  "mcpServers": {
    "search": {
      "command": "mcp-search",
      "args": [],
      "env": {
        "ELASTICSEARCH_URL": "http://localhost:9200",
        "ELASTICSEARCH_USER": "elastic",
        "ELASTICSEARCH_PASSWORD": "changeme"
      }
    }
  }
}
```

## Usage Examples

### Full-text search
```
"Find all articles about kubernetes deployment"
→ search(query="kubernetes deployment", index="articles", limit=10)
```

### Semantic search (RAG)
```
"Find content similar to this paragraph about microservices"
→ semantic_search(vector=[0.1, 0.2, ...], index="docs", top_k=5)
```

### Autocomplete
```
"Suggest completions for 'kube'"
→ suggest(query="kube", index="articles")
→ ["kubernetes", "kubectl", "kubelet"]
```

### Index a document
```
"Add this blog post to the search index"
→ index_document(index="articles", document={"title":"...", "body":"...", "tags":[...]})
```

### Explain ranking
```
"Why did this doc rank #3?"
→ explain_ranking(query="kubernetes", document_id="doc-123", index="articles")
```

## License

Apache-2.0

---

Part of the [ADK-Rust Enterprise](https://enterprise.adk-rust.com) MCP server ecosystem.

Built with ❤️ by [Zavora AI](https://zavora.ai)

## rmcp and MCP compatibility

This server is built with [`rmcp` 3.1.2](https://github.com/modelcontextprotocol/rust-sdk/releases/tag/rmcp-v3.1.2) and requires Rust 1.94.1 or newer. The rmcp 3 rollout retains legacy MCP initialization compatibility and targets MCP protocol revisions `2025-11-25` and `2026-07-28`.

## MCP 2026-07-28 rollout (P5 read-heavy)

This server uses `rmcp` 3.1.2 and `adk-mcp-sdk` 0.2 with a minimum supported
Rust version of **1.94.1**. It accepts stateless MCP 2026 requests with
per-request protocol, client identity, and capability metadata while retaining
the legacy MCP 2025-11-25 initialize flow for ordinary tools.

- **Tasks:** `get_index`, `create_index`, `index_document`, `get_index_stats`
- **MRTR approvals:** `delete_document`
- **Discovery and routing:** rmcp serves on-demand discovery and validates the
  per-request protocol envelope; HTTP deployments can route with `Mcp-Method`
  and `Mcp-Name`. The packaged binary currently uses stdio.
- **Caching:** `tools/list` returns a public `ttlMs` of 60,000 for MCP 2026;
  rmcp omits the cache fields for legacy clients.
- **Deprecated extensions:** this server does not add new Roots, Sampling, or
  dynamic client-registration dependencies.

Protected tools require `MCP_REQUEST_STATE_KEY` with at least 32 high-entropy
bytes. All replicas must share that key so sealed approval state can resume on
another instance. Approval state is bound to the client identity, tool, and
arguments and expires after two minutes. Missing identity, invalid state,
rejection, or legacy protocol use fails closed. Task records are process-local
for the current stdio runtime; use a durable task store before deploying the
server behind scale-to-zero HTTP infrastructure.
