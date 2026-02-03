# CurhatIn SDK (Backend)

<div align="center">
  <h3>Privacy-First AI Mental Health Support Engine</h3>
  <p>High-performance, stateless backend for secure and empathetic AI interactions.</p>
</div>

---

## 📖 Overview

**CurhatIn SDK** is the backend engine powering the CurhatIn platform. Built with **Rust** and **Axum**, it is designed for rapid, concurrent handling of chat sessions while maintaining strict privacy standards. This SDK abstracts the complexity of LLM interactions (via OpenRouter), data sanitation, and state management, providing a clean REST API for the frontend.

## ✨ Features

- **🚀 High Performance**: Built on Rust's Actix/Axum ecosystem for blazing fast response times.
- **🔒 Privacy-First**: Stateless architecture ensures no personal identifiable information (PII) is permanently stored.
- **🧠 Modular AI Engine**: Easily swappable LLM providers via OpenRouter integration.
- **📄 Auto-Documentation**: Integrated Swagger/OpenAPI UI for live API testing (`/swagger-ui`).
- **🛡️ Ethical Guardrails**: System prompts designed to prevent medical diagnosis and prioritize user safety.

## 🛠️ Tech Stack

- **Language**: Rust
- **Framework**: [Axum](https://github.com/tokio-rs/axum)
- **Runtime**: Tokio
- **Database**: MongoDB (for transient session storage, if enabled)
- **Documentation**: Utoipa (OpenAPI)
- **AI Provider**: OpenRouter

## 🚀 Getting Started

### Prerequisites

- **Rust** (latest stable)
- **Docker** & **Docker Compose**
- **MongoDB** instance (local or Atlas)

### Installation

1.  **Clone the repository**
    ```bash
    git clone https://github.com/yourusername/ai-mental-chatbot-sdk.git
    cd ai-mental-chatbot-sdk
    ```

2.  **Configure Environment**
    Copy the example configuration:
    ```bash
    cp .env.example .env
    ```
    Update the `.env` file with your credentials:
    ```env
    PORT=3000
    MONGODB_URI=mongodb://localhost:27017
    MONGODB_DATABASE=curhatin_db
    OPENROUTER_API_KEY=your_key_here
    OPENROUTER_MODEL=deepseek/deepseek-charter:free
    ```

### Running Locally

```bash
cargo run
```
The server will start at `http://localhost:3000`.

### Running with Docker

```bash
docker build -t curhatin-sdk .
docker run -p 3000:3000 --env-file .env curhatin-sdk
```

## 📚 API Documentation

Once the server is running, you can explore the full API documentation interactively:

- **Swagger UI**: [http://localhost:3000/swagger-ui](http://localhost:3000/swagger-ui)
- **OpenAPI Spec**: [http://localhost:3000/api-docs/openapi.json](http://localhost:3000/api-docs/openapi.json)

## 🤝 Contributing

We welcome contributions! Please check `docs/PRODUCT_WORKFLOW.md` (legacy context) for understanding the original project scope.

1.  Fork the Project
2.  Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3.  Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4.  Push to the Branch (`git push origin feature/AmazingFeature`)
5.  Open a Pull Request

## 📄 License

Distributed under the MIT License. See `LICENSE` for more information.
