mod db;
mod embeddings;
mod rag;

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use chrono::Utc;
use db::{AppDatabase, KnowledgeDocument};
use embeddings::EmbeddingService;
use rag::RagService;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

// ===== Configuration =====
struct AppConfig {
    openrouter_api_key: String,
    openrouter_model: String,
}

// ===== Shared State =====
struct AppState {
    config: AppConfig,
    db: AppDatabase,
    embedding_service: EmbeddingService,
}

// ===== Request/Response Types =====
#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    message: String,
}

#[derive(Debug, Deserialize)]
struct ChatRequest {
    message: String,
    #[serde(default)]
    conversation_history: Vec<Message>,
}

#[derive(Debug, Serialize)]
struct ChatResponse {
    response: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sources: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct IngestRequest {
    title: String,
    content: String,
    category: String,
}

#[derive(Debug, Serialize)]
struct IngestResponse {
    success: bool,
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

// ===== OpenRouter Types =====
#[derive(Debug, Serialize)]
struct OpenRouterRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Debug, Deserialize)]
struct OpenRouterResponse {
    choices: Vec<OpenRouterChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterChoice {
    message: OpenRouterMessage,
}

#[derive(Debug, Deserialize)]
struct OpenRouterMessage {
    content: String,
}

// ===== Mental Health System Prompt =====
const SYSTEM_PROMPT: &str = r#"You are a compassionate mental wellness companion called MindJournal Assistant. Your role is to provide a safe space for reflection and emotional support.

## Your Approach:
- Listen with genuine empathy and reflect back what users share
- Ask thoughtful, open-ended questions to help users explore their feelings
- Summarize and validate emotions without judgment
- Use warm, supportive language that feels natural and caring
- Be present and patient, not rushing to solve problems

## Important Boundaries (NEVER violate these):
1. NEVER diagnose mental health conditions (no "you might have depression/anxiety")
2. NEVER prescribe treatments, medications, or specific therapies
3. NEVER give direct advice like "You should..." or "You must..."
4. NEVER claim to be a therapist, doctor, or medical professional
5. If someone expresses thoughts of self-harm or suicide, respond with:
   - Acknowledge their pain with compassion
   - Gently encourage them to reach out to crisis support:
     "I hear that you're going through something really difficult. Please consider reaching out to a crisis helpline - in Indonesia you can contact Into The Light (119 ext 8) or Yayasan Pulih (021-788-42580). You deserve support from people who can truly help."

## Response Style:
- Keep responses warm but concise (2-4 paragraphs max)
- Use reflective statements: "It sounds like...", "I hear that..."
- Ask one thoughtful question at a time to encourage deeper reflection
- Validate feelings before exploring further
- Respond in the same language the user writes in (Indonesian or English)

Remember: You are a mirror for reflection, not a problem-solver. Help users discover their own insights."#;

// ===== Handlers =====
async fn health_check(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // Check MongoDB connection
    let db_status = match state.db.ping().await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };
    
    Json(HealthResponse {
        status: "ok".to_string(),
        message: format!("AI Mental Chatbot Backend is running. MongoDB: {}", db_status),
    })
}

async fn chat(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ChatRequest>,
) -> impl IntoResponse {
    // Validate input
    if payload.message.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ChatResponse {
                response: String::new(),
                error: Some("Message cannot be empty".to_string()),
                sources: None,
            }),
        );
    }

    // Create RAG service and retrieve context
    let rag_service = RagService::new(state.db.clone(), EmbeddingService::new(state.config.openrouter_api_key.clone()));
    
    let (augmented_prompt, sources) = match rag_service.retrieve_context(&payload.message, 3).await {
        Ok(context) => {
            let sources: Vec<String> = context.iter().map(|d| d.title.clone()).collect();
            let prompt = rag_service.augment_prompt(SYSTEM_PROMPT, &context);
            (prompt, if sources.is_empty() { None } else { Some(sources) })
        }
        Err(e) => {
            tracing::warn!("RAG retrieval failed, using base prompt: {}", e);
            (SYSTEM_PROMPT.to_string(), None)
        }
    };

    // Build messages with system prompt and conversation history
    let mut messages = vec![Message {
        role: "system".to_string(),
        content: augmented_prompt,
    }];

    // Add conversation history (limited to last 10 messages for context)
    let history_start = if payload.conversation_history.len() > 10 {
        payload.conversation_history.len() - 10
    } else {
        0
    };
    messages.extend(payload.conversation_history[history_start..].to_vec());

    // Add current user message
    messages.push(Message {
        role: "user".to_string(),
        content: payload.message,
    });

    // Call OpenRouter API
    let client = reqwest::Client::new();
    let openrouter_request = OpenRouterRequest {
        model: state.config.openrouter_model.clone(),
        messages,
        max_tokens: 500,
        temperature: 0.7,
    };

    let response = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", state.config.openrouter_api_key))
        .header("Content-Type", "application/json")
        .header("HTTP-Referer", "https://mindjournal.app")
        .header("X-Title", "MindJournal")
        .json(&openrouter_request)
        .send()
        .await;

    match response {
        Ok(res) => {
            if res.status().is_success() {
                match res.json::<OpenRouterResponse>().await {
                    Ok(openrouter_response) => {
                        let ai_response = openrouter_response
                            .choices
                            .first()
                            .map(|c| c.message.content.clone())
                            .unwrap_or_else(|| "I'm here to listen. How are you feeling today?".to_string());

                        (
                            StatusCode::OK,
                            Json(ChatResponse {
                                response: ai_response,
                                error: None,
                                sources,
                            }),
                        )
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse OpenRouter response: {}", e);
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ChatResponse {
                                response: String::new(),
                                error: Some("Failed to process AI response".to_string()),
                                sources: None,
                            }),
                        )
                    }
                }
            } else {
                let status = res.status();
                let error_text = res.text().await.unwrap_or_default();
                tracing::error!("OpenRouter API error: {} - {}", status, error_text);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ChatResponse {
                        response: String::new(),
                        error: Some("AI service temporarily unavailable".to_string()),
                        sources: None,
                    }),
                )
            }
        }
        Err(e) => {
            tracing::error!("Failed to call OpenRouter API: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ChatResponse {
                    response: String::new(),
                    error: Some("Failed to connect to AI service".to_string()),
                    sources: None,
                }),
            )
        }
    }
}

/// Ingest a new document into the knowledge base
async fn ingest_document(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<IngestRequest>,
) -> impl IntoResponse {
    // Validate input
    if payload.content.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(IngestResponse {
                success: false,
                id: String::new(),
                error: Some("Content cannot be empty".to_string()),
            }),
        );
    }

    // Generate embedding for the content
    let embedding = match state.embedding_service.generate_embedding(&payload.content).await {
        Ok(emb) => emb,
        Err(e) => {
            tracing::error!("Failed to generate embedding: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(IngestResponse {
                    success: false,
                    id: String::new(),
                    error: Some(format!("Failed to generate embedding: {}", e)),
                }),
            );
        }
    };

    // Create document
    let doc_id = Uuid::new_v4().to_string();
    let document = KnowledgeDocument {
        id: doc_id.clone(),
        title: payload.title,
        content: payload.content,
        category: payload.category,
        embedding,
        created_at: Utc::now(),
    };

    // Insert into MongoDB
    let collection = state.db.knowledge_collection();
    match collection.insert_one(document).await {
        Ok(_) => {
            tracing::info!("Ingested document: {}", doc_id);
            (
                StatusCode::CREATED,
                Json(IngestResponse {
                    success: true,
                    id: doc_id,
                    error: None,
                }),
            )
        }
        Err(e) => {
            tracing::error!("Failed to insert document: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(IngestResponse {
                    success: false,
                    id: String::new(),
                    error: Some(format!("Failed to store document: {}", e)),
                }),
            )
        }
    }
}

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ai_mental_chatbot_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let openrouter_api_key = std::env::var("OPENROUTER_API_KEY")
        .expect("OPENROUTER_API_KEY must be set");
    
    let config = AppConfig {
        openrouter_api_key: openrouter_api_key.clone(),
        openrouter_model: std::env::var("OPENROUTER_MODEL")
            .unwrap_or_else(|_| "openai/gpt-4o-mini".to_string()),
    };

    // Connect to MongoDB
    let mongodb_uri = std::env::var("MONGODB_URI")
        .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    let mongodb_database = std::env::var("MONGODB_DATABASE")
        .unwrap_or_else(|_| "mental_chatbot".to_string());
    
    let db = AppDatabase::connect(&mongodb_uri, &mongodb_database)
        .await
        .expect("Failed to connect to MongoDB");

    // Create embedding service
    let embedding_service = EmbeddingService::new(openrouter_api_key);

    // Create shared state
    let state = Arc::new(AppState {
        config,
        db,
        embedding_service,
    });

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/chat", post(chat))
        .route("/api/ingest", post(ingest_document))
        .layer(cors)
        .with_state(state);

    // Start server
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    tracing::info!("🚀 Server running on http://localhost:{}", port);

    axum::serve(listener, app).await.unwrap();
}
