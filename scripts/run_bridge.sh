#!/bin/bash
# Start the Trading Bridge Server

set -e

echo "🧬 Starting Trading Creatures Bridge..."

# Check if GROQ_API_KEY is set
if [ -z "$GROQ_API_KEY" ]; then
    echo "❌ Error: GROQ_API_KEY environment variable not set"
    echo "   Get your free API key from: https://console.groq.com/keys"
    echo "   Then run: export GROQ_API_KEY='your-key-here'"
    exit 1
fi

echo "✅ GROQ_API_KEY found"

# Navigate to rust bridge directory
cd "$(dirname "$0")/../src/rust-bridge"

# Build and run
echo "🔨 Building Rust bridge..."
cargo build --release

echo "🚀 Starting server on http://localhost:3030"
cargo run --release
