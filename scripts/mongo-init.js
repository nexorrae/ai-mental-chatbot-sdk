// MongoDB initialization script
// This runs when the container is first created

// Switch to the target database
db = db.getSiblingDB(process.env.MONGO_INITDB_DATABASE || 'mental_chatbot');

// Create the knowledge collection with an index
db.createCollection('knowledge');

// Create index on embedding field for better performance
// Note: For full vector search, use MongoDB Atlas with $vectorSearch
db.knowledge.createIndex({ "category": 1 });
db.knowledge.createIndex({ "created_at": -1 });

print('MongoDB initialization complete!');
