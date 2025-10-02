# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

MemeForAnyone is an open-source, high-efficiency, easy-to-deploy meme recommendation system built in Rust. It uses semantic matching through embedding models, VLM for image tagging, and vector similarity search to recommend relevant memes based on user queries.

**GitHub Repository**: https://github.com/DanielZhangyc/MemeForAnyone
**License**: Open Source (Apache 2.0 or MIT)
**Target Users**: Developers deploying meme recommendation services
**Use Cases**: QQ Bots, websites, input methods, embodied robots (emotion display)
**Original Prototype**: [MemeMeow](https://github.com/MemeMeow-Studio/MemeMeow)

---

## Complete Tech Stack (2025)

### Backend Framework
- **Axum 0.8** - Rust web framework
  - Tokio-native async runtime
  - 100% safe Rust (`#![forbid(unsafe_code)]`)
  - Lowest memory footprint (12-15MB)
  - Most adopted Rust web framework in 2025
  - Excellent Tower middleware ecosystem

### Databases & Storage
- **Vector Database**: Qdrant 1.15+ (Rust-native)
  - Primary vector similarity search
  - 40-470 QPS with cosine similarity optimization
  - gRPC API via `qdrant-client` Rust crate
  - Alternative: PostgreSQL + pgvector (via SeaORM) for <1M vectors

- **Relational Database**: PostgreSQL 16 with pgvector extension
  - ORM: SeaORM 1.1+ (native async, pgvector support since March 2025)
  - Meme metadata, user management, API keys
  - Connection pooling: Built into SeaORM (max_connections=20)

- **Object Storage**: S3-compatible storage (flexible deployment)
  - Rust client: `aws-sdk-s3` (supports any S3-compatible service)
  - Supports: AWS S3, MinIO, Cloudflare R2, Alibaba OSS, etc.
  - Users configure their preferred storage provider via environment variables
  - Pre-signed URL support for secure temporary access

- **Caching**:
  - L1 (In-memory): Moka 0.12 with TTL/TTI support
  - L2 (Distributed): Redis 7 with `fred` Rust client (async-first)

### ML Model Stack (Chinese Market Optimized)

#### Primary Stack
- **Embedding**: Alibaba Qwen3-Embedding-8B via SiliconFlow ($0.04/1M tokens)
  - Fallback: Local BGE-M3 via FastEmbed-rs (self-hosted, free)
  - International fallback: OpenAI text-embedding-3-small

- **VLM (Image Tagging)**: Alibaba Qwen-VL-Plus (~$0.002/image)
  - Extracts main subject + emotion → text tags
  - Fallback: Zhipu GLM-4V-Flash (free tier) or GPT-4o-mini

- **Reranking**: Local BGE-reranker-v2-m3 (ONNX, self-hosted)
  - 69.02 multilingual ranking score
  - Zero API cost, complete privacy

#### ML Infrastructure
- **Local Inference**: `ort` (ONNX Runtime) 1.13.2
  - 3-5x faster than PyTorch, 60-80% less memory
  - GPU acceleration support (CUDA, TensorRT)
  - Alternative: `candle` (HuggingFace Rust framework)

- **Embedding Library**: `fastembed-rs` with BGE-M3 quantized models
- **Vector Math**: `ndarray` 0.16.x + `similarity` crate (optimized cosine)
- **Image Processing**: `image` crate 0.25.8 (resize to 224x224 for VLM input)

### HTTP & Networking
- **HTTP Client**: `reqwest` 0.12.x (async, connection pooling)
- **Async Runtime**: Tokio 1.x (multi-threaded)
- **Reverse Proxy**: Caddy 2 (automatic HTTPS via Let's Encrypt)

### Observability
- **Logging**: `tracing` 0.1 + `tracing-subscriber` (structured, JSON output)
- **Metrics**: Prometheus + `axum-prometheus` (custom metrics)
- **Tracing**: OpenTelemetry + Jaeger (distributed tracing)
- **Monitoring**: Grafana dashboards

### Configuration & Security
- **Config Management**: `figment` 0.10 (TOML + env vars)
- **Secrets**: Docker secrets (dev) / Kubernetes secrets (prod)
- **Security Auditing**: `cargo-audit` + `cargo-deny` in CI/CD
- **Password Hashing**: `argon2` 0.5

### Deployment
- **Containerization**: Docker multi-stage builds
  - Builder: `rust:1.83-bookworm-slim`
  - Runtime: `gcr.io/distroless/cc-debian12` (minimal attack surface)
  - Build optimization: `cargo-chef` (5x faster rebuilds)
- **Orchestration**: Docker Compose (dev/small prod), Kubernetes (scale)

---

## Core System Architecture

### Data Flow

```
User Query → Embedding API → Vector (768D)
                ↓
        Qdrant Similarity Search (cosine)
                ↓
        Top K Meme IDs → PostgreSQL Metadata
                ↓
        MinIO Pre-signed URLs → User Response
```

### Image Ingestion Pipeline

```
Image Upload → MinIO Storage
      ↓
VLM API (Qwen-VL-Plus) → Extract Tags + Emotion
      ↓
Tags + Metadata → PostgreSQL (SeaORM)
      ↓
Tags → Embedding API → Vector (768D)
      ↓
Qdrant Vector Storage (with payload metadata)
```

### Caching Strategy

```
Request → L1 Cache (Moka, 5min TTL)
            ↓ (miss)
        L2 Cache (Redis, 5min TTL)
            ↓ (miss)
        Database/Qdrant Search
            ↓
        Populate L2 & L1 → Return
```

---

## API Endpoints

### `GET /query`
Search for similar memes based on text query.

**Parameters**:
- `content` (string, required): Search query text (max 500 chars)
- `n` (int, optional): Number of results (default: 5, max: 100)

**Response**:
```json
{
  "code": 200,
  "data": [
    "https://cdn.example.com/meme1.jpg",
    "https://cdn.example.com/meme2.jpg"
  ],
  "msg": "",
  "uuid": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": 1704067200
}
```

**Implementation Notes**:
- Rate limit: 100 requests/minute per IP (Caddy)
- Cache TTL: 5 minutes (Moka L1 + Redis L2)
- Query processing: <100ms target latency
- Vector search: <30ms p95 latency (Qdrant)

### `POST /pull`
Trigger image ingestion from configured storage.

**Authentication**: API key via `X-API-Key` header

**Request Body**:
```json
{
  "source": "s3://bucket/path" | "local:///path" | "oss://bucket/path",
  "batch_size": 100
}
```

**Response**:
```json
{
  "code": 200,
  "msg": "Pull initiated",
  "job_id": "job_123",
  "estimated_images": 1500
}
```

**Implementation Notes**:
- Background job processing (Tokio task)
- Progress tracking via Redis
- VLM batch processing: 32 images/batch
- Error handling: Failed images logged for retry

### `GET /health`
Health check endpoint for Docker/K8s.

**Response**:
```json
{
  "status": "healthy",
  "timestamp": "2025-01-15T12:00:00Z",
  "services": {
    "postgres": "ok",
    "qdrant": "ok",
    "redis": "ok",
    "minio": "ok"
  }
}
```

### `GET /metrics`
Prometheus metrics endpoint.

**Custom Metrics**:
- `meme_queries_total` - Total query count
- `vector_search_duration_seconds` - Search latency histogram
- `embedding_api_errors_total` - ML API error counter
- `cache_hit_ratio` - L1/L2 cache hit rate

---

## Development Commands

### Initial Setup
```bash
# Clone repository
git clone https://github.com/your-org/MemeForAnyone.git
cd MemeForAnyone

# Install Rust toolchain (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install development tools
cargo install cargo-watch cargo-audit cargo-deny sea-orm-cli

# Start services (PostgreSQL, Qdrant, MinIO, Redis)
docker compose up -d

# Run database migrations
sea-orm-cli migrate up

# Create Qdrant collection
cargo run --bin init-qdrant
```

### Development Workflow
```bash
# Run in development mode with auto-reload
cargo watch -x run

# Run specific binary
cargo run --bin meme-api

# Run tests
cargo test

# Run tests with coverage
cargo install cargo-tarpaulin
cargo tarpaulin --out Html

# Check code formatting
cargo fmt

# Lint code
cargo clippy -- -D warnings

# Security audit
cargo audit

# Dependency license/security check
cargo deny check
```

### Database Operations
```bash
# Create new migration
sea-orm-cli migrate generate create_memes_table

# Apply migrations
sea-orm-cli migrate up

# Rollback migration
sea-orm-cli migrate down

# Generate entities from database
sea-orm-cli generate entity -o src/entities
```

### Docker Operations
```bash
# Build Docker image
docker build -t meme-api:latest .

# Run with docker-compose
docker compose up -d

# View logs
docker compose logs -f api

# Rebuild after code changes
docker compose up -d --build api

# Stop all services
docker compose down

# Clean volumes (CAUTION: deletes data)
docker compose down -v
```

### Production Deployment
```bash
# Build optimized release
cargo build --release

# Build multi-arch Docker image
docker buildx build --platform linux/amd64,linux/arm64 -t meme-api:v1.0.0 .

# Push to registry
docker push ghcr.io/your-org/meme-api:v1.0.0

# Deploy with Docker Compose (production)
docker compose -f docker-compose.prod.yml up -d

# View Prometheus metrics
open http://localhost:9090

# View Grafana dashboards
open http://localhost:3000
```

---

## Configuration

### Environment Variables

```bash
# Server
APP__SERVER__HOST=0.0.0.0
APP__SERVER__PORT=8080
APP__SERVER__WORKERS=16

# Database
APP__DATABASE__URL=postgresql://user:pass@postgres:5432/meme_db
APP__DATABASE__MAX_CONNECTIONS=50

# Qdrant
APP__QDRANT__URL=http://qdrant:6334
APP__QDRANT__API_KEY=your_api_key
APP__QDRANT__COLLECTION_NAME=memes

# ML APIs (Chinese providers)
APP__EMBEDDING__PROVIDER=siliconflow  # siliconflow | local | openai
APP__EMBEDDING__API_KEY=your_key
APP__EMBEDDING__MODEL=Qwen/Qwen3-Embedding-8B

APP__VLM__PROVIDER=qwen  # qwen | zhipu | openai
APP__VLM__API_KEY=your_key
APP__VLM__MODEL=qwen-vl-plus

# Storage
APP__STORAGE__MINIO_ENDPOINT=http://minio:9000
APP__STORAGE__MINIO_ACCESS_KEY=minioadmin
APP__STORAGE__MINIO_SECRET_KEY=minioadmin
APP__STORAGE__BUCKET_NAME=memes

# Redis
APP__REDIS__URL=redis://:password@redis:6379
APP__REDIS__POOL_SIZE=10

# Logging
RUST_LOG=info,meme_api=debug,tower_http=debug
```

### Configuration Files

- `config/default.toml` - Base configuration
- `config/development.toml` - Dev overrides
- `config/production.toml` - Prod overrides (minimal, use env vars)
- `.env` - Local environment variables (gitignored)
- `.env.example` - Template for environment variables

---

## Performance Targets

### API Latency (p95)
- `/query` endpoint: <100ms (including cache lookup)
- Vector search (Qdrant): <30ms
- Embedding API call: <50ms
- PostgreSQL metadata fetch: <10ms

### Throughput
- Single instance: 100-400 QPS
- Horizontal scaling: Linear up to 10 instances
- Max concurrent connections: 1000 per instance

### Cache Hit Rates
- L1 (Moka): Target 70-80%
- L2 (Redis): Target 85-90%
- Combined: >90% cache hit rate

### Resource Usage (Single Instance)
- Memory: 1-2GB (with local ONNX models)
- CPU: 1-2 cores baseline, 4+ cores for VLM processing
- Disk: 50GB for models + storage
- Network: 1Gbps recommended

---

## Security Considerations

### Authentication & Authorization
- API key authentication for `/pull` endpoint
- Rate limiting: 100 req/min per IP (Caddy layer)
- CORS configuration for web clients
- Input validation: Max query length 500 chars, n ≤ 100

### Data Privacy (Chinese Market)
- **PIPL Compliance**: All user data stored within China
- **Data Residency**: Use Alibaba Cloud/local hosting
- **ML Processing**: Prefer local ONNX models (zero data leakage)
- **Logging**: Mask sensitive data (user IDs, API keys)

### Infrastructure Security
- **Container**: Distroless runtime (no shell, minimal packages)
- **User**: Non-root (UID 65532)
- **Secrets**: Docker secrets / K8s secrets (never in env vars)
- **TLS**: Automatic HTTPS via Caddy Let's Encrypt
- **Dependencies**: `cargo-audit` + `cargo-deny` in CI/CD

### Input Validation
```rust
// Example validation
use validator::Validate;

#[derive(Validate)]
struct QueryParams {
    #[validate(length(min = 1, max = 500))]
    content: String,

    #[validate(range(min = 1, max = 100))]
    n: usize,
}
```

---

## Testing Strategy

### Unit Tests
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_vector_similarity

# Run with output
cargo test -- --nocapture
```

### Integration Tests
```rust
// tests/integration_test.rs
#[tokio::test]
async fn test_query_endpoint() {
    let app = create_test_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/query?content=happy%20cat&n=5")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
```

### Load Testing
```bash
# Install k6
brew install k6  # macOS

# Run load test
k6 run tests/load-test.js

# Target: 100 RPS for 5 minutes
k6 run --vus 10 --duration 5m tests/load-test.js
```

---

## Deployment Architecture

### Development (Local)
```
Developer Machine
├── Rust API (cargo run)
├── PostgreSQL (Docker)
├── Qdrant (Docker)
├── MinIO (Docker)
└── Redis (Docker)
```

### Production (Single Server)
```
VPS/Cloud Server (4 CPU, 8GB RAM)
├── Caddy (reverse proxy, HTTPS)
├── Rust API (Docker, 2 replicas)
├── PostgreSQL (Docker + volume)
├── Qdrant (Docker + volume)
├── MinIO (Docker + volume)
├── Redis (Docker + AOF persistence)
├── Prometheus (monitoring)
└── Grafana (dashboards)
```

### Production (Kubernetes)
```
K8s Cluster
├── Ingress (Nginx/Traefik)
├── API Deployment (HPA: 2-10 replicas)
├── PostgreSQL StatefulSet (with persistent volumes)
├── Qdrant StatefulSet (with persistent volumes)
├── MinIO StatefulSet (or external OSS)
├── Redis Cluster
└── Monitoring Stack (Prometheus Operator + Grafana)
```

---

## Cost Estimates (100K queries/month, Chinese Market)

### ML APIs
- Embedding (Alibaba Qwen3-8B): $0.20/month
- VLM (Qwen-VL-Plus, 20K images): $40/month
- Reranking (Local BGE): $0 (self-hosted)
- **Total ML**: ~$40-50/month

### Infrastructure (Self-hosted)
- VPS (4 CPU, 8GB RAM, 100GB SSD): $30-50/month
- Bandwidth (1TB): $10-20/month
- **Total Infrastructure**: ~$40-70/month

### Alternative: Cloud-managed (Alibaba Cloud)
- ECS (2 CPU, 4GB): ¥100-200/month (~$14-28)
- OSS Storage (100GB): ¥10/month (~$1.50)
- CDN (1TB): ¥100/month (~$14)
- **Total**: ~$30-45/month

**Grand Total**: $70-120/month for 100K queries

---

## Troubleshooting

### Common Issues

**Issue**: Slow vector search
**Solution**:
- Enable Qdrant scalar quantization (75% memory reduction)
- Check HNSW index configuration (m=16, ef_construct=100)
- Monitor CPU usage during search

**Issue**: Out of memory
**Solution**:
- Reduce connection pool size (max_connections=10)
- Enable Qdrant binary quantization (96% memory reduction)
- Use swap for large model loading

**Issue**: High API latency
**Solution**:
- Check cache hit rate (should be >90%)
- Monitor embedding API response time
- Increase Redis TTL for stable queries
- Add Moka in-memory cache layer

**Issue**: Database connection timeout
**Solution**:
- Increase `connect_timeout` in SeaORM config
- Check PostgreSQL max_connections setting
- Monitor connection pool exhaustion

**Issue**: Docker build too slow
**Solution**:
- Use `cargo-chef` for dependency caching
- Enable BuildKit cache: `DOCKER_BUILDKIT=1`
- Use `sccache` for incremental compilation

---

## Contributing

### Code Style
- Follow Rust standard formatting: `cargo fmt`
- Pass all clippy lints: `cargo clippy -- -D warnings`
- Write tests for new features
- Update documentation

### Pull Request Checklist
- [ ] All tests pass (`cargo test`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Security audit clean (`cargo audit`)
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Performance impact assessed

---

## Resources

### Documentation
- [Axum Guide](https://docs.rs/axum/latest/axum/)
- [SeaORM Book](https://www.sea-ql.org/SeaORM/)
- [Qdrant Documentation](https://qdrant.tech/documentation/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)

### ML Model Providers
- [SiliconFlow Platform](https://www.siliconflow.com/)
- [Alibaba Cloud Qwen](https://www.alibabacloud.com/en/solutions/generative-ai/qwen)
- [FastEmbed-rs](https://github.com/Anush008/fastembed-rs)

### Community
- Rust China Conf: https://rustcc.cn/
- SeaQL Discord
- Qdrant Discord

---

## License

(To be determined - suggest Apache 2.0 or MIT for open source)
