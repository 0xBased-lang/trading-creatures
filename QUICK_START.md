# Quick Start: Get Running in One Day

This guide gets you from zero to a working signal validation system in ~6-8 hours.

---

## Prerequisites

- VPS with 2GB+ RAM, 2+ CPU cores
- SSH access to your VPS
- Basic command line knowledge

---

## Step 1: Initial Setup (30 minutes)

### SSH into your VPS
```bash
ssh user@your-vps-ip
```

### Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustc --version  # Verify installation
```

### Install Python 3.10
```bash
# Check if already installed
python3 --version

# If not 3.10+, install:
sudo apt update
sudo apt install python3.10 python3.10-venv python3-pip -y
```

### Get Groq API Key (FREE)
```bash
# 1. Go to: https://console.groq.com/keys
# 2. Sign up with email (free)
# 3. Create API key
# 4. Copy the key (starts with 'gsk_')

# Add to environment
echo 'export GROQ_API_KEY="gsk_your_key_here"' >> ~/.bashrc
source ~/.bashrc
```

---

## Step 2: Clone Repositories (10 minutes)

```bash
# Create project directory
mkdir ~/trading-creatures-project
cd ~/trading-creatures-project

# Clone Creature framework
git clone https://github.com/getbasedai/creature.git
cd creature
cargo build --release  # This takes 5-10 minutes
cd ..

# Clone Moon Dev agents
git clone https://github.com/moondevonyt/moon-dev-ai-agents.git
cd moon-dev-ai-agents
python3.10 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
cd ..

# Clone this integration repo
git clone https://github.com/0xBased-lang/trading-creatures.git
```

---

## Step 3: Modify Creature for Groq (2-3 hours)

This is the most involved step. We're adding Groq API support to Creature.

### Option A: Automated (if we provide scripts)
```bash
cd ~/trading-creatures-project/trading-creatures/scripts
./patch_creature_for_groq.sh
```

### Option B: Manual (more learning)

**Create:** `creature/src/api/groq_client.rs`

```rust
use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::env;

#[derive(Debug, Serialize)]
struct GroqRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct GroqResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

pub struct GroqClient {
    client: Client,
    api_key: String,
    model: String,
}

impl GroqClient {
    pub fn new() -> Self {
        let api_key = env::var("GROQ_API_KEY")
            .expect("GROQ_API_KEY must be set");

        Self {
            client: Client::new(),
            api_key,
            model: "llama-3.1-70b-versatile".to_string(),
        }
    }

    pub async fn generate(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        let request = GroqRequest {
            model: self.model.clone(),
            messages: vec![
                Message {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                }
            ],
            temperature: 0.7,
            max_tokens: 1000,
        };

        let response = self.client
            .post("https://api.groq.com/openai/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;

        let groq_response: GroqResponse = response.json().await?;

        Ok(groq_response.choices[0].message.content.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_groq_client() {
        let client = GroqClient::new();
        let result = client.generate("Say hello!").await;
        assert!(result.is_ok());
    }
}
```

**Modify:** `creature/src/api/mod.rs`
```rust
pub mod groq_client;  // Add this line
pub mod openrouter;
```

**Modify:** `creature/Cargo.toml`
```toml
[dependencies]
# ... existing dependencies ...
reqwest = { version = "0.11", features = ["json"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

**Test it:**
```bash
cd ~/trading-creatures-project/creature
cargo test groq_client
```

If test passes, you're good! ✅

---

## Step 4: Add Trading Bridge API (2-3 hours)

**Create:** `creature/src/api/trading_bridge.rs`

```rust
use axum::{
    routing::{get, post},
    Router,
    Json,
    extract::State,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Deserialize)]
pub struct SignalRequest {
    signal_id: String,
    signal_type: String,
    data: serde_json::Value,
    context: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ValidationResponse {
    signal_id: String,
    validation_result: String,
    consensus_score: f64,
    confidence: f64,
    aggregate_reasoning: String,
}

pub async fn start_trading_bridge_server() {
    let app = Router::new()
        .route("/api/v1/signal/validate", post(validate_signal))
        .route("/api/v1/colony/status", get(get_colony_status))
        .route("/health", get(health_check));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3030")
        .await
        .unwrap();

    println!("🚀 Trading bridge server listening on http://127.0.0.1:3030");

    axum::serve(listener, app).await.unwrap();
}

async fn validate_signal(
    Json(request): Json<SignalRequest>
) -> Json<ValidationResponse> {
    // TODO: Implement actual validation logic
    // For now, return mock response

    Json(ValidationResponse {
        signal_id: request.signal_id,
        validation_result: "VALID".to_string(),
        consensus_score: 0.75,
        confidence: 0.80,
        aggregate_reasoning: "Mock validation - implementation pending".to_string(),
    })
}

async fn get_colony_status() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "colony_size": 5,
        "uptime_seconds": 3600,
    }))
}

async fn health_check() -> &'static str {
    "OK"
}
```

**Update** `creature/Cargo.toml`:
```toml
[dependencies]
axum = "0.7"
tower = "0.4"
```

**Update** `creature/src/main.rs`:
```rust
mod api;

#[tokio::main]
async fn main() {
    // Start trading bridge server
    api::trading_bridge::start_trading_bridge_server().await;
}
```

**Test it:**
```bash
cd ~/trading-creatures-project/creature
cargo run --release

# In another terminal:
curl http://localhost:3030/health
# Should return: OK

curl -X POST http://localhost:3030/api/v1/signal/validate \
  -H "Content-Type: application/json" \
  -d '{"signal_id":"test1","signal_type":"whale","data":{},"context":{}}'
# Should return JSON response
```

---

## Step 5: Create Python Client (1 hour)

**Create:** `moon-dev-ai-agents/src/utils/creature_client.py`

```python
import aiohttp
import asyncio
from typing import Dict, Optional
from dataclasses import dataclass

@dataclass
class ValidationResult:
    signal_id: str
    decision: str
    consensus_score: float
    confidence: float
    reasoning: str

class CreatureClient:
    def __init__(self, host="localhost", port=3030):
        self.base_url = f"http://{host}:{port}"

    async def validate_signal(self, signal_type: str, data: Dict) -> ValidationResult:
        payload = {
            "signal_id": f"{signal_type}_{int(time.time())}",
            "signal_type": signal_type,
            "data": data,
            "context": {}
        }

        async with aiohttp.ClientSession() as session:
            async with session.post(
                f"{self.base_url}/api/v1/signal/validate",
                json=payload
            ) as response:
                result = await response.json()
                return ValidationResult(
                    signal_id=result["signal_id"],
                    decision=result["validation_result"],
                    consensus_score=result["consensus_score"],
                    confidence=result["confidence"],
                    reasoning=result["aggregate_reasoning"]
                )

# Test it
async def test_client():
    client = CreatureClient()
    result = await client.validate_signal(
        signal_type="whale",
        data={"wallet": "0x123", "amount": 500000}
    )
    print(f"Result: {result}")

if __name__ == "__main__":
    import time
    asyncio.run(test_client())
```

**Test it:**
```bash
cd ~/trading-creatures-project/moon-dev-ai-agents
source venv/bin/activate
python src/utils/creature_client.py
# Should print validation result
```

---

## Step 6: Test Integration (30 minutes)

**Create test script:** `test_integration.py`

```python
import asyncio
from src.utils.creature_client import CreatureClient

async def test_whale_signal():
    client = CreatureClient()

    # Simulate whale signal
    signal_data = {
        "wallet_address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
        "token": "BTC",
        "amount_usd": 500000,
        "current_price": 43250.50,
        "volume_24h": 28500000000
    }

    print("🐋 Testing whale signal validation...")
    result = await client.validate_signal("whale", signal_data)

    print(f"\n✅ Validation complete!")
    print(f"   Decision: {result.decision}")
    print(f"   Consensus: {result.consensus_score:.1%}")
    print(f"   Reasoning: {result.reasoning}")

if __name__ == "__main__":
    asyncio.run(test_whale_signal())
```

**Run test:**
```bash
# Terminal 1: Start Creature server
cd ~/trading-creatures-project/creature
cargo run --release

# Terminal 2: Run test
cd ~/trading-creatures-project/moon-dev-ai-agents
source venv/bin/activate
python test_integration.py
```

If you see validation results, you're done with Day 1! 🎉

---

## What You've Built

✅ Creature framework running with Groq API
✅ REST API for signal validation
✅ Python client for Moon Dev agents
✅ End-to-end signal flow working

**Current limitations:**
- Validation is mocked (returns dummy data)
- No actual Creature colony logic yet
- No memory/learning yet

**Next steps (Week 2+):**
- Implement actual validation logic
- Add Creature colony with specialized cells
- Integrate with real Moon Dev whale agent
- Add memory and learning

---

## Troubleshooting

### Rust compilation fails
```bash
# Update Rust
rustup update

# Clean and rebuild
cargo clean
cargo build --release
```

### Port 3030 already in use
```bash
# Find and kill process
lsof -i :3030
kill -9 <PID>

# Or change port in trading_bridge.rs
```

### Python can't connect to Creature
```bash
# Check if Creature is running
curl http://localhost:3030/health

# Check firewall
sudo ufw status
sudo ufw allow 3030
```

### Groq API errors
```bash
# Verify API key
echo $GROQ_API_KEY

# Test with curl
curl -X POST "https://api.groq.com/openai/v1/chat/completions" \
  -H "Authorization: Bearer $GROQ_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"model":"llama-3.1-70b-versatile","messages":[{"role":"user","content":"Hello!"}]}'
```

---

## Cost Check

After Day 1, check your Groq usage:
```bash
# Go to: https://console.groq.com/usage
# Should show: <100 requests (well within free tier)
```

---

## Next Session

Once this is working, we'll implement:
1. Real validation logic with LLM analysis
2. Creature colony with 5 specialized cells
3. Memory system for learning
4. Integration with actual whale_agent.py

**Estimated time:** 3-4 more sessions (12-16 hours total)

---

Ready to start? Let me know if you hit any issues!
