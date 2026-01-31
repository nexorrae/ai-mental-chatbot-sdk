use mongodb::{Client, Collection, Database};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Knowledge document stored in MongoDB with vector embedding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeDocument {
    #[serde(rename = "_id")]
    pub id: String,
    pub content: String,
    pub title: String,
    pub category: String,
    pub embedding: Vec<f64>,
    pub created_at: DateTime<Utc>,
}

/// Conversation message for history tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

/// MongoDB database wrapper
#[derive(Clone)]
pub struct AppDatabase {
    pub client: Client,
    pub db: Database,
}

impl AppDatabase {
    /// Connect to MongoDB
    pub async fn connect(uri: &str, database_name: &str) -> Result<Self, mongodb::error::Error> {
        let client = Client::with_uri_str(uri).await?;
        let db = client.database(database_name);
        
        tracing::info!("Connected to MongoDB database: {}", database_name);
        
        Ok(Self { client, db })
    }
    
    /// Get the knowledge documents collection
    pub fn knowledge_collection(&self) -> Collection<KnowledgeDocument> {
        self.db.collection("knowledge")
    }
    
    /// Check connection health
    pub async fn ping(&self) -> Result<(), mongodb::error::Error> {
        self.db.run_command(mongodb::bson::doc! { "ping": 1 }).await?;
        Ok(())
    }
}
