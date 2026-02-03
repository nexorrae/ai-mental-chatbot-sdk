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
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;
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
#[derive(Debug, Serialize, ToSchema)]
struct HealthResponse {
    status: String,
    message: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ChatRequest {
    #[schema(example = "Halo, saya merasa cemas")]
    message: String,
    #[schema(example = "general")]
    category: Option<String>,
    #[serde(default)]
    #[schema(default)]
    conversation_history: Vec<Message>,
}

#[derive(Debug, Serialize, ToSchema)]
struct ChatResponse {
    response: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sources: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Message {
    #[schema(example = "user")]
    role: String,
    #[schema(example = "saya merasa cemas")]
    content: String,
}

#[derive(Debug, Deserialize, ToSchema)]
struct IngestRequest {
    title: String,
    content: String,
    category: String,
}

#[derive(Debug, Serialize, ToSchema)]
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
const SYSTEM_PROMPT_GENERAL: &str = r#"You are a compassionate mental wellness companion called Curhatin Assistant. Your role is to provide a safe space for reflection and emotional support.

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



const SYSTEM_PROMPT_GENERAL: &str = r#"You are a compassionate mental wellness companion called Curhatin Assistant. Your role is to provide a safe space for reflection and emotional support.

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
- If the user discusses a specific topic (Career, Romance, etc.), maintain this general supportive stance but acknowledge the context.

Remember: You are a mirror for reflection, not a problem-solver. Help users discover their own insights."#;

const SYSTEM_PROMPT_CAREER: &str = r#"You are a supportive career confident and mental wellness companion called Curhatin Assistant. Your role is to listen to career-related concerns (burnout, office politics, direction, failure) and help the user reflect.

## Your Approach:
- Focus on the user's feelings about their work, not just the technical details.
- Validate feelings of stress, inadequacy, or confusion.
- Ask questions that help them clarify their values and what they want from their career.
- Avoid giving specific career advice (e.g., "apply to this job"), instead help them uncover their own answers.

## Important Boundaries:
- adhere to the same safety and non-medical boundaries as the General prompt.

## Response Style:
- Professional yet empathetic tone.
- Use phrases like "It sounds like this situation is draining you..." or "What does success look like to you in this context?"
"#;

const SYSTEM_PROMPT_ROMANCE: &str = r#"You are a compassionate relationship confidant and mental wellness companion called Curhatin Assistant. Your role is to listen to concerns about love, dating, breakups, and loneliness.

## Your Approach:
- Create a safe space to vent about heartbreaks or relationship anxiety.
- Validate feelings of rejection, love, or confusion without taking sides (if they complain about a partner).
- Encourage healthy communication and self-respect.
- Help them distinguish between what they can control and what they cannot.

## Important Boundaries:
- adhere to the same safety and non-medical boundaries as the General prompt.

## Response Style:
- Warm, gentle, and understanding.
- Use phrases like "It hurts to feel disconnected..." or "What do you need most from a partner right now?"
"#;

const SYSTEM_PROMPT_FAMILY: &str = r#"You are a compassionate listener for family matters, called Curhatin Assistant. Your role is to support users dealing with family conflict, distance, or expectations.

## Your Approach:
- Validate the complexity of family dynamics (guilt, obligation, love).
- Help the user establish healthy boundaries in their mind.
- Encourage empathy for themselves and family members (where safe).

## Important Boundaries:
- adhere to the same safety and non-medical boundaries as the General prompt.

## Response Style:
- Respectful of cultural nuances regarding family.
- Gentle and grounding.
"#;

const SYSTEM_PROMPT_SELF_DEVELOPMENT: &str = r#"You are a growth-oriented companion called Curhatin Assistant. Your role is to support the user in their journey of self-improvement, habits, and self-worth.

## Your Approach:
- Celebrate small wins and intentions.
- Help them explore "why" they want to change or grow.
- Be a sounding board for their goals, helping them break down overwhelming feelings.
- Challenge negative self-talk gently.

## Important Boundaries:
- adhere to the same safety and non-medical boundaries as the General prompt.

## Response Style:
- Encouraging, motivating (but not "toxic positivity"), and reflective.
"#;

fn get_system_prompt(category: Option<&str>) -> String {
    match category.unwrap_or("general").to_lowercase().as_str() {
        "karir" | "career" => format!("{}\n\n{}", SYSTEM_PROMPT_GENERAL, SYSTEM_PROMPT_CAREER),
        "asmara" | "romance" | "love" => format!("{}\n\n{}", SYSTEM_PROMPT_GENERAL, SYSTEM_PROMPT_ROMANCE),
        "keluarga" | "family" => format!("{}\n\n{}", SYSTEM_PROMPT_GENERAL, SYSTEM_PROMPT_FAMILY),
        "pengembangan diri" | "self development" | "growth" => format!("{}\n\n{}", SYSTEM_PROMPT_GENERAL, SYSTEM_PROMPT_SELF_DEVELOPMENT),
        _ => SYSTEM_PROMPT_GENERAL.to_string(),
    }
}

// ===== ApiDoc =====
#[derive(OpenApi)]
#[openapi(
    paths(health_check, chat, ingest_document),
    components(
        schemas(HealthResponse, ChatRequest, ChatResponse, Message, IngestRequest, IngestResponse)
    ),
    tags(
        (name = "ai-mental-chatbot", description = "AI Mental Chatbot Backend API")
    )
)]
struct ApiDoc;

// ===== Handlers =====

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Server is healthy", body = HealthResponse)
    )
)]
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

/// Chat with AI
#[utoipa::path(
    post,
    path = "/api/chat",
    request_body = ChatRequest,
    responses(
        (status = 200, description = "Chat response", body = ChatResponse),
        (status = 400, description = "Bad request", body = ChatResponse),
        (status = 500, description = "Internal server error")
    )
)]
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
            let prompt = rag_service.augment_prompt(&get_system_prompt(payload.category.as_deref()), &context);
            (prompt, if sources.is_empty() { None } else { Some(sources) })
        }
        Err(e) => {
            tracing::warn!("RAG retrieval failed, using base prompt: {}", e);
            (get_system_prompt(payload.category.as_deref()), None)
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
        .header("HTTP-Referer", "https://Curhatin.app")
        .header("X-Title", "Curhatin")
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

/// Ingest a document
#[utoipa::path(
    post,
    path = "/api/ingest",
    request_body = IngestRequest,
    responses(
        (status = 201, description = "Document ingested", body = IngestResponse),
        (status = 400, description = "Bad request", body = IngestResponse)
    )
)]
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
    
    // Connect to MongoDB with retry logic
    let mut db = None;
    let max_retries = 5;
    for i in 1..=max_retries {
        match AppDatabase::connect(&mongodb_uri, &mongodb_database).await {
            Ok(database) => {
                db = Some(database);
                break;
            }
            Err(e) => {
                if i == max_retries {
                    tracing::error!("Failed to connect to MongoDB after {} attempts: {}", max_retries, e);
                    panic!("Failed to connect to MongoDB: {}", e);
                }
                tracing::warn!("Failed to connect to MongoDB (attempt {}/{}): {}. Retrying in 5s...", i, max_retries, e);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    }
    let db = db.expect("Failed to connect to MongoDB");

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
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/health", get(health_check))
        .route("/api/chat", post(chat))
        .route("/api/ingest", post(ingest_document))
        .layer(cors)
        .with_state(state);

    // Start server
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let port = if port.is_empty() { "3000".to_string() } else { port };
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    tracing::info!("🚀 Server running on http://localhost:{}", port);
    tracing::info!("📜 Swagger UI available at http://localhost:{}/swagger-ui", port);

    axum::serve(listener, app).await.unwrap();
}
