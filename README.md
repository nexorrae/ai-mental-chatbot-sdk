# AI Mental Chatbot SDK (Backend)

The backend service for the AI Mental Health Chatbot, built with Rust and Axum. This service handles chat interactions, integrates with OpenRouter (LLM), and manages data persistence via MongoDB.

## 🛠 Tech Stack
- **Language:** Rust
- **Framework:** [Axum](https://github.com/tokio-rs/axum)
- **Database:** MongoDB
- **Async Runtime:** Tokio
- **HTTP Client:** Reqwest
- **Logging:** Tracing

## 🚀 Key Features
- **REST API:** Endpoints for chat and health checks.
- **RAG Integration:** (Retrieval-Augmented Generation) Support for context-aware responses (see `rag.rs`).
- **Database Integration:** Async MongoDB operations (see `db.rs`).
- **OpenRouter Support:** Middleware for interacting with LLM providers.

## 📦 Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [Docker](https://www.docker.com/) (optional, for containerization)
- A running MongoDB instance

## ⚙️ Configuration
Copy the `.env.example` file to `.env` and configure your keys:

```bash
cp .env.example .env
```

### Environment Variables
| Variable | Description | Default |
|----------|-------------|---------|
| `PORT` | Server listening port | `3000` |
| `OPENROUTER_API_KEY` | **Required** API key for OpenRouter | - |
| `MONGODB_URI` | Connection string for MongoDB | `mongodb://localhost:27017...` |
| `RUST_LOG` | Logging level | `info` |

## 🏃‍♂️ Running Locally

1. **Install Dependencies & Build:**
   ```bash
   cargo build
   ```

2. **Run the Server:**
   ```bash
   cargo run
   ```

3. **Check Status:**
   Visit `http://localhost:3000/health` (or your configured port).

## 🐳 Docker Support
Build and run with Docker:

```bash
docker build -t mental-chatbot-sdk .
docker run -p 3000:3000 --env-file .env mental-chatbot-sdk
```

## 📂 Project Structure
- `src/main.rs`: Application entry point and route configuration.
- `src/db.rs`: Database connection and helper functions.
- `src/rag.rs`: Logic for Retrieval-Augmented Generation.
- `src/embeddings.rs`: Handling vector embeddings.