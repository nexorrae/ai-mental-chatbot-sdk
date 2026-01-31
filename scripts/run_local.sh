#!/bin/bash
# scripts/run_local.sh
# Check if MongoDB is running and start the Rust backend locally

# Ensure we are in the project root
cd "$(dirname "$0")/.."

echo "🔍 Checking system status..."

# 1. Check if .env exists
if [ ! -f .env ]; then
    echo "❌ Error: .env file not found!"
    echo "   Please copy .env.example to .env and configure it."
    exit 1
fi

# 2. Check Payload Connection (MongoDB)
# Mac/Linux lsof check
if command -v lsof >/dev/null; then
    if ! lsof -i :27017 >/dev/null; then
        echo "⚠️  MongoDB is NOT running on port 27017."
        echo "   The backend needs MongoDB to function."
        read -p "   start the database via Docker? (y/N) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            docker compose up -d mongodb
            echo "⏳ Waiting for MongoDB to initialize..."
            sleep 5
        else
            echo "   Proceeding without starting MongoDB (might fail)..."
        fi
    else
        echo "✅ MongoDB is running."
    fi
fi

# 3. Run the project
echo "🚀 Starting Rust Backend (Cargo)..."
echo "   Listening on http://localhost:3000"
echo "   Press Ctrl+C to stop."
echo ""

# Use cargo watch if available (hot reload), otherwise cargo run
if command -v cargo-watch >/dev/null; then
    echo "ℹ️  Using 'cargo watch' for hot reloading..."
    cargo watch -x run
else
    cargo run
fi
