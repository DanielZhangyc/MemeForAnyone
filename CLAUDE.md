# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository

**GitHub:** https://github.com/DanielZhangyc/MemeForAnyone

## Project Overview

MemeForAnyone is a **backend server** for meme recommendation, based on the VVQuest/MemeMeow project (https://github.com/MemeMeow-Studio/MemeMeow). The goal is to create an easy-to-deploy, efficient, and flexible meme recommendation service that can be integrated into various applications.

**Key Features:**
- Semantic matching using Embedding + Rerank
- Meme tag matching using Search model + VLM (Vision-Language Model)
- Support for external storage connections
- Manual/automatic meme library updates

**Use Cases:**
- QQ Bot integration
- Web applications
- Input method integration
- Embodied robots (for emotion display)

## Tech Stack

### Core Technologies
- **Backend Framework**: Axum 0.7+ (Tokio-based async web framework)
- **ML Inference**: ONNX Runtime (ort crate 2.0+)
- **Vector Database**: Qdrant (Rust-native vector search)
- **Storage**: object_store 0.11+ (multi-cloud abstraction: S3/MinIO/Local)
- **Database**: PostgreSQL with pgvector (metadata storage)
- **Container**: Docker (multi-stage builds with cargo-chef)

### ML Models (via API)
- **Embedding**: BGE-M3 (BAAI, 1024-dim, Chinese+English) - API endpoint
- **Reranker**: BGE Reranker v2-m3 (based on Gemma-2B) - API endpoint
- **VLM**: Chinese-CLIP Base (OFA-Sys, auto-tagging) - API endpoint

**Deployment Options**:
1. External API services (recommended for start): Jina AI, OpenAI, Voyage AI, etc.
2. Self-hosted API: Deploy models with FastAPI/vLLM in separate container
3. Local ONNX: Embedded inference (future optimization)

### Key Dependencies
```toml
axum = "0.7"                        # Web framework
tokio = { version = "1", features = ["full"] }
tower-governor = "0.7"              # Rate limiting
reqwest = "0.11"                    # HTTP client (for ML API calls)
qdrant-client = "1.9"               # Vector database
object_store = "0.11"               # Storage abstraction
sqlx = "0.8"                        # Database access
image = "0.25"                      # Image processing
validator = "0.18"                  # Input validation
tracing = "0.1"                     # Structured logging
serde_json = "1.0"                  # JSON serialization
```

## Project Status

This is an early-stage project. The codebase is being built from scratch.

**Current Phase**: Architecture design and planning complete
**Next Steps**: Project initialization and core implementation

## API Design

### Public Endpoints (No Auth Required)
```
GET  /api/v1/search              # Search memes by query
GET  /api/v1/memes/{id}          # Get meme details
GET  /health                     # Health check
```

### Admin Endpoints (Token Auth Required)
```
POST   /api/v1/admin/memes/sync  # Sync local meme library
PATCH  /api/v1/admin/memes/{id}  # Update meme tags/metadata
DELETE /api/v1/admin/memes/{id}  # Delete meme
POST   /api/v1/admin/reindex     # Rebuild vector index
```

### API Example
**Search Request:**
```bash
GET /api/v1/search?q=悲伤的猫&n=10&tags=猫,可爱&format=detailed
```

**Response:**
```json
{
  "code": 200,
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "url": "https://cdn.example.com/memes/a1b2c3.jpg",
      "tags": ["猫", "悲伤", "可爱"],
      "score": 0.95
    }
  ],
  "msg": "",
  "meta": { "took_ms": 87, "total": 2 }
}
```

## Security Model

### Authentication
- **Public APIs**: No auth, IP-based rate limiting (100 req/min)
- **Admin APIs**: Bearer token authentication
- **Token Format**: `meme_sk_{32-char-random}` (SHA-256 hashed in DB)

### Rate Limiting
| Endpoint | Limit | Scope |
|----------|-------|-------|
| Search | 100/min | Per-IP |
| Admin CRUD | 30/min | Per-Token |
| Sync/Reindex | 5/hour | Per-Token |

### Security Features
- Token-based admin authentication
- Input validation with `validator` crate
- Image content validation (format/size/dimensions)
- Audit logging for all admin operations
- CORS and security headers via Tower middleware

## Architecture

### System Architecture
```
┌─────────────────────────────────────────────────┐
│              Axum Web Server (Port 8080)        │
│  ┌──────────────┐  ┌──────────────────────────┐ │
│  │ Public API   │  │ Admin API (Token Auth)   │ │
│  │ - Search     │  │ - Sync  - Update - Delete│ │
│  └──────┬───────┘  └──────────┬───────────────┘ │
└─────────┼──────────────────────┼──────────────────┘
          │                      │
          ▼                      ▼
┌─────────────────────────────────────────────────┐
│           Recommendation Engine                  │
│         (HTTP Client for ML APIs)                │
│  ┌──────────┐  ┌──────────┐  ┌──────────────┐  │
│  │Embedding │  │ Reranker │  │     VLM      │  │
│  │   API    │  │   API    │  │     API      │  │
│  │  20-50ms │  │  30-80ms │  │  50-150ms    │  │
│  └────┬─────┘  └────┬─────┘  └──────┬───────┘  │
└───────┼─────────────┼────────────────┼──────────┘
        │             │                │
        │             │                │
        ▼             ▼                ▼
┌─────────────────────────────────────────────────┐
│          External ML API Services                │
│  (Jina AI / OpenAI / Self-hosted FastAPI)       │
└─────────────────────────────────────────────────┘
        │             │                │
        ▼             ▼                ▼
┌─────────────────────────────────────────────────┐
│              Data Storage Layer                  │
│  ┌──────────────┐  ┌─────────────────────────┐  │
│  │   Qdrant     │  │    PostgreSQL           │  │
│  │ (Vector DB)  │  │  (Metadata + Tokens)    │  │
│  │  10-30ms     │  │      5-10ms             │  │
│  └──────────────┘  └─────────────────────────┘  │
│                                                   │
│  ┌──────────────────────────────────────────┐   │
│  │   Object Store (S3/MinIO/Local)          │   │
│  │   (Meme Image Files)                      │   │
│  └──────────────────────────────────────────┘   │
└─────────────────────────────────────────────────┘
```

### Project Structure
```
meme-for-anyone/
├── Cargo.toml                  # Dependencies
├── Dockerfile                  # Multi-stage build
├── docker-compose.yml          # Local dev environment
├── .env.example                # Environment variables template
├── config/
│   └── default.toml           # Configuration template
├── src/
│   ├── main.rs                # Entry point
│   ├── config.rs              # Config management
│   ├── api/                   # HTTP API layer
│   │   ├── routes.rs          # Route definitions
│   │   ├── handlers/          # Request handlers
│   │   │   ├── search.rs      # Search endpoint
│   │   │   ├── admin.rs       # Admin endpoints
│   │   │   └── health.rs      # Health check
│   │   └── dto.rs             # Data transfer objects
│   ├── ml/                    # ML API clients
│   │   ├── client.rs          # Base HTTP client
│   │   ├── embedding.rs       # Embedding API
│   │   ├── reranker.rs        # Reranker API
│   │   └── vlm.rs             # VLM API
│   ├── search/                # Vector search
│   │   ├── vector_store.rs    # Qdrant integration
│   │   └── semantic.rs        # Search logic
│   ├── storage/               # File storage
│   │   ├── trait.rs           # Storage abstraction
│   │   ├── s3.rs              # S3/MinIO
│   │   └── local.rs           # Local filesystem
│   ├── middleware/            # Auth, rate limiting
│   │   ├── auth.rs            # Token authentication
│   │   └── rate_limit.rs      # Rate limiting
│   ├── db/                    # Database
│   │   ├── models.rs          # Database models
│   │   └── queries.rs         # SQL queries
│   └── domain/                # Domain models
│       └── meme.rs            # Meme entity
└── tests/
    ├── integration/
    └── fixtures/
```

### Recommendation Flow
```
User Query "悲伤的猫咪"
    ↓
1. Embedding API Call                    → 20-50ms
   └─ POST to embedding service
    ↓
2. Vector Search (Qdrant)                → 10-30ms
   └─ Returns Top-100 candidates
    ↓
3. Tag Filtering (optional)              → 5ms
    ↓
4. Reranker API Call                     → 30-80ms
   └─ POST query + Top-100 docs
   └─ Returns Top-10 results
    ↓
Total Latency: 65-165ms (P95 < 200ms)
```

### ML API Configuration
```toml
# config/default.toml
[ml_apis]
# Option 1: External services (Jina AI, OpenAI, etc.)
embedding_url = "https://api.jina.ai/v1/embeddings"
embedding_api_key = "${JINA_API_KEY}"
embedding_model = "jina-embeddings-v2-base-zh"

reranker_url = "https://api.jina.ai/v1/rerank"
reranker_api_key = "${JINA_API_KEY}"
reranker_model = "jina-reranker-v2-base-multilingual"

vlm_url = "https://api.openai.com/v1/chat/completions"
vlm_api_key = "${OPENAI_API_KEY}"
vlm_model = "gpt-4-vision-preview"

# Option 2: Self-hosted
# embedding_url = "http://ml-service:8000/embed"
# reranker_url = "http://ml-service:8000/rerank"
# vlm_url = "http://ml-service:8000/tag"
```

### Performance Targets
| Metric | Target | Hardware |
|--------|--------|----------|
| Query Latency (P95) | < 200ms | 4-8 core CPU |
| Throughput | 50-100 RPS | 8GB RAM |
| Vector DB Size | 1M vectors | ~2GB RAM |
| Main Service RAM | ~1-2GB | Rust backend |
| Total RAM | 4-8GB | Recommended |

**Notes**:
- ML inference offloaded to external APIs (no local GPU/model memory needed)
- Latency includes network overhead for API calls
- Self-hosted ML services require additional 4-8GB RAM

## Development Commands

### Setup
```bash
# Clone repository
git clone https://github.com/DanielZhangyc/MemeForAnyone.git
cd MemeForAnyone

# Start with Docker Compose
docker-compose up -d

# Check health
curl http://localhost:8080/health
```

### Development
```bash
# Build
cargo build --release

# Run locally
cargo run

# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy
```

### Docker Operations
```bash
# Build image
docker build -t meme-for-anyone:latest .

# Run container
docker run -p 8080:8080 meme-for-anyone:latest

# View logs
docker logs -f meme-api
```

### Admin Operations
```bash
# Generate admin token
docker exec meme-api /app/meme-cli token create --name "Admin Token"

# Sync local memes
curl -X POST http://localhost:8080/api/v1/admin/memes/sync \
  -H "Authorization: Bearer meme_sk_xxx" \
  -H "Content-Type: application/json" \
  -d '{"source": "/data/memes", "auto_tag": true}'

# Check sync status
curl http://localhost:8080/api/v1/admin/tasks/{task_id} \
  -H "Authorization: Bearer meme_sk_xxx"
```