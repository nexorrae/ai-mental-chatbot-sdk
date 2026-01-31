use crate::db::{AppDatabase, KnowledgeDocument};
use crate::embeddings::{cosine_similarity, EmbeddingService};
use futures::stream::TryStreamExt;

/// Retrieved document with similarity score
#[derive(Debug, Clone)]
pub struct RetrievedDocument {
    pub content: String,
    pub title: String,
    pub category: String,
    #[allow(dead_code)]
    pub similarity: f64,
}

/// RAG (Retrieval-Augmented Generation) service
pub struct RagService {
    db: AppDatabase,
    embedding_service: EmbeddingService,
}

impl RagService {
    pub fn new(db: AppDatabase, embedding_service: EmbeddingService) -> Self {
        Self { db, embedding_service }
    }
    
    /// Retrieve relevant documents based on query similarity
    pub async fn retrieve_context(
        &self,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<RetrievedDocument>, String> {
        // Generate embedding for the query
        let query_embedding = self.embedding_service.generate_embedding(query).await?;
        
        // Fetch all documents (for local similarity search)
        // Note: For production with MongoDB Atlas, use $vectorSearch aggregation
        let collection = self.db.knowledge_collection();
        let cursor = collection
            .find(mongodb::bson::doc! {})
            .await
            .map_err(|e| format!("Failed to query documents: {}", e))?;
        
        let documents: Vec<KnowledgeDocument> = cursor
            .try_collect()
            .await
            .map_err(|e| format!("Failed to collect documents: {}", e))?;
        
        // Calculate similarity and rank
        let mut scored_docs: Vec<(KnowledgeDocument, f64)> = documents
            .into_iter()
            .map(|doc| {
                let similarity = cosine_similarity(&query_embedding, &doc.embedding);
                (doc, similarity)
            })
            .collect();
        
        // Sort by similarity descending
        scored_docs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Take top K results with minimum similarity threshold
        let min_similarity = 0.3;
        let results: Vec<RetrievedDocument> = scored_docs
            .into_iter()
            .take(top_k)
            .filter(|(_, sim)| *sim >= min_similarity)
            .map(|(doc, similarity)| RetrievedDocument {
                content: doc.content,
                title: doc.title,
                category: doc.category,
                similarity,
            })
            .collect();
        
        tracing::debug!("Retrieved {} relevant documents for query", results.len());
        Ok(results)
    }
    
    /// Augment the system prompt with retrieved context
    pub fn augment_prompt(&self, base_prompt: &str, context: &[RetrievedDocument]) -> String {
        if context.is_empty() {
            return base_prompt.to_string();
        }
        
        let context_text: String = context
            .iter()
            .enumerate()
            .map(|(i, doc)| {
                format!(
                    "---\nDocument {} ({}): {}\n{}\n",
                    i + 1,
                    doc.category,
                    doc.title,
                    doc.content
                )
            })
            .collect();
        
        format!(
            "{}\n\n## Reference Knowledge Base\nUse the following information to provide accurate, helpful responses when relevant:\n\n{}\n---\n\nRemember: Only reference this information if it's relevant to the user's question. Always prioritize empathetic listening.",
            base_prompt,
            context_text
        )
    }
}
