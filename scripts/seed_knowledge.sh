#!/bin/bash
# Seed the knowledge base with initial documents
# Usage: ./seed_knowledge.sh

API_URL="${API_URL:-http://localhost:3000}"
SEED_FILE="data/knowledge_seed.json"

echo "🌱 Seeding knowledge base..."

# Check if jq is installed
if ! command -v jq &> /dev/null; then
    echo "Error: jq is required but not installed."
    echo "Install with:"
    echo "  MacOS:  brew install jq"
    echo "  Ubuntu: sudo apt-get install jq"
    exit 1
fi

# Check if curl is installed
if ! command -v curl &> /dev/null; then
    echo "Error: curl is required but not installed."
    echo "Install with:"
    echo "  MacOS:  brew install curl"
    echo "  Ubuntu: sudo apt-get install curl"
    exit 1
fi

# Check if seed file exists
if [ ! -f "$SEED_FILE" ]; then
    echo "Error: Seed file not found: $SEED_FILE"
    exit 1
fi

# Read and ingest each document
count=0
jq -c '.[]' "$SEED_FILE" | while read -r doc; do
    title=$(echo "$doc" | jq -r '.title')
    
    response=$(curl -s -X POST "$API_URL/api/ingest" \
        -H "Content-Type: application/json" \
        -d "$doc")
    
    success=$(echo "$response" | jq -r '.success')
    
    if [ "$success" = "true" ]; then
        echo "✅ Ingested: $title"
        ((count++))
    else
        error=$(echo "$response" | jq -r '.error // "Unknown error"')
        echo "❌ Failed: $title - $error"
    fi
done

echo ""
echo "🎉 Knowledge base seeding complete!"
