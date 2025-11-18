# Trading Creatures - VPS Setup Guide

## System Requirements

✅ **Your Setup:** VPS with CPU (no GPU needed)
✅ **Languages:** Rust + Python
✅ **LLM Strategy:** Groq API (free tier, CPU-friendly)

---

## Phase 1: Signal Validation System (Weeks 1-4)

### Goal
Reduce false signals by 30-50% using Creature colony validation

### Architecture
```
Moon Dev Agents (Python) → Creature Colony (Rust + Groq) → Validated Signals (Python)
```

---

## Installation Steps

### 1. Install Rust (if not already installed)
```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Verify
rustc --version
cargo --version
```

### 2. Clone Repositories
```bash
cd ~
mkdir trading-creatures-project
cd trading-creatures-project

# Clone Creature framework
git clone https://github.com/getbasedai/creature.git

# Clone Moon Dev agents
git clone https://github.com/moondevonyt/moon-dev-ai-agents.git

# Create our integration layer
mkdir creature-trading-bridge
cd creature-trading-bridge
```

### 3. Set Up Groq API (FREE)
```bash
# Get free API key from: https://console.groq.com/keys
# Sign up (free), create API key

# Add to environment
echo 'export GROQ_API_KEY="your-groq-api-key-here"' >> ~/.bashrc
source ~/.bashrc

# Verify
echo $GROQ_API_KEY
```

### 4. Modify Creature for Groq
We'll create a Groq client to replace OpenRouter in Creature.

**File: `creature/src/api/groq_client.rs`** (we'll create this)
```rust
// Groq API client for fast, cheap LLM inference
// - Free tier: 14,400 requests/day
// - Speed: 500+ tokens/second (fastest available)
// - Models: Llama 3.1 70B, Mixtral, Gemma
```

### 5. Set Up Python Environment
```bash
cd ~/trading-creatures-project/moon-dev-ai-agents

# Use system Python or pyenv
python3.10 -m venv venv
source venv/bin/activate

# Install Moon Dev requirements
pip install -r requirements.txt

# Install additional packages for Creature bridge
pip install websockets aiohttp python-dotenv
```

---

## Project Structure

```
~/trading-creatures-project/
├── creature/                          # Original Creature framework
│   ├── src/
│   │   ├── api/
│   │   │   ├── groq_client.rs        # NEW: Groq API client
│   │   │   └── trading_bridge.rs     # NEW: Trading signal API
│   │   ├── systems/
│   │   │   └── signal_validator.rs   # NEW: Signal validation logic
│   │   └── main.rs
│   └── Cargo.toml
│
├── moon-dev-ai-agents/                # Original Moon Dev
│   ├── src/
│   │   ├── agents/
│   │   │   ├── whale_agent.py        # Will modify
│   │   │   ├── sentiment_agent.py    # Will modify
│   │   │   └── ...
│   │   └── utils/
│   │       └── creature_client.py    # NEW: Python client for Creature
│   └── requirements.txt
│
└── creature-trading-bridge/          # Our integration layer
    ├── config/
    │   └── colony_config.json        # Colony configuration
    ├── scripts/
    │   ├── start_creature.sh         # Launch Creature server
    │   └── test_bridge.py            # Test connection
    └── docs/
        └── SETUP.md                  # This file
```

---

## Configuration

### Groq API Configuration
```bash
# ~/.env
GROQ_API_KEY=gsk_xxxxxxxxxxxxx
GROQ_MODEL=llama-3.1-70b-versatile  # Fast, smart, free
GROQ_TEMPERATURE=0.7
GROQ_MAX_TOKENS=1000
```

### Creature Colony Configuration
```json
{
  "colony_size": 5,
  "signal_types": ["whale", "sentiment", "volume", "liquidation"],
  "consensus_threshold": 0.7,
  "cell_specializations": [
    {
      "cell_id": 1,
      "role": "historical_pattern_matcher",
      "thought_dna": {
        "emergence": 20,
        "coherence": 60,
        "resilience": 50,
        "intelligence": 80,
        "efficiency": 40,
        "integration": 50
      }
    },
    {
      "cell_id": 2,
      "role": "volume_analyzer",
      "thought_dna": {
        "emergence": 30,
        "coherence": 70,
        "resilience": 60,
        "intelligence": 50,
        "efficiency": 80,
        "integration": 40
      }
    },
    {
      "cell_id": 3,
      "role": "correlation_detector",
      "thought_dna": {
        "emergence": 50,
        "coherence": 40,
        "resilience": 50,
        "intelligence": 60,
        "efficiency": 50,
        "integration": 90
      }
    },
    {
      "cell_id": 4,
      "role": "risk_assessor",
      "thought_dna": {
        "emergence": 10,
        "coherence": 90,
        "resilience": 80,
        "intelligence": 70,
        "efficiency": 60,
        "integration": 50
      }
    },
    {
      "cell_id": 5,
      "role": "anomaly_detector",
      "thought_dna": {
        "emergence": 90,
        "coherence": 30,
        "resilience": 70,
        "intelligence": 60,
        "efficiency": 50,
        "integration": 40
      }
    }
  ]
}
```

### Moon Dev Integration Configuration
```python
# src/config/creature_config.py

CREATURE_ENABLED = True
CREATURE_HOST = "localhost"
CREATURE_PORT = 3030
CREATURE_TIMEOUT = 5.0  # seconds

# Signal validation thresholds
VALIDATION_THRESHOLDS = {
    "whale": 0.70,      # 70% consensus required
    "sentiment": 0.65,   # 65% consensus required
    "volume": 0.75,      # 75% consensus required
    "liquidation": 0.60  # 60% consensus required
}

# Fallback behavior if Creature unavailable
FALLBACK_MODE = "conservative"  # or "aggressive" or "disable"
```

---

## Resource Requirements

### CPU Usage
- **Creature (Rust):** ~5-10% CPU per validation (very efficient)
- **Groq API:** All compute happens on Groq servers (your VPS just makes HTTP requests)
- **Total VPS impact:** Minimal (~10-15% CPU)

### Memory Usage
- **Creature:** ~200-500 MB RAM (with 5-cell colony)
- **Moon Dev:** ~500-800 MB RAM (existing)
- **Total:** ~1-1.5 GB RAM required

### Network/API Limits
- **Groq Free Tier:** 14,400 requests/day = 600/hour = 10/minute
- **Our Usage (5 cells validating 50 signals/day):** 250 requests/day
- **Headroom:** 98% unused capacity on free tier!

### Storage
- **Creature state:** ~50-100 MB
- **Signal logs:** ~10 MB/day
- **Total:** ~500 MB for 30 days of data

---

## Next Steps

1. **Set up VPS environment** (Rust, Python, Groq API)
2. **Modify Creature for Groq** (create groq_client.rs)
3. **Build trading bridge** (REST + WebSocket API)
4. **Create Python client** (creature_client.py)
5. **Integrate whale agent** (first signal type)
6. **Test and measure** (compare before/after)
7. **Expand to other signals** (sentiment, volume, etc.)

---

## Performance Expectations

### Week 1: Foundation
- Creature running with Groq
- WebSocket bridge functional
- Basic validation working

### Week 2: Whale Integration
- Whale agent connected
- First validated signals
- Data collection started

### Week 3: Optimization
- Tune consensus thresholds
- Adjust cell DNA configurations
- Improve response times

### Week 4: Measurement
- Compare signal quality (before/after)
- Measure false positive reduction
- Calculate P&L improvement

### Week 5+: Expansion
- Add sentiment validation
- Add volume validation
- Begin strategy evolution (Phase 2)

---

## Troubleshooting

### Groq API Rate Limits
```bash
# If you hit rate limits (unlikely on free tier)
# Implement request queuing:
- Add 100ms delay between requests
- Use request batching
- Upgrade to Groq Pro ($0.05/1M tokens)
```

### Latency Issues
```bash
# Creature validation should be <500ms
# If slower:
- Check Groq API latency (should be <200ms)
- Reduce max_tokens in Groq requests
- Use faster Groq model (llama-3.1-8b)
```

### Memory Issues
```bash
# If VPS RAM constrained:
- Reduce colony size (5 → 3 cells)
- Decrease max_memory_size in Creature
- Enable memory compression more aggressively
```

---

## Cost Analysis (Your VPS Setup)

### Monthly Costs
```
VPS: $existing (no change)
Groq API: $0 (free tier sufficient)
Additional storage: $0 (minimal)
----------------------------------
Total additional cost: $0/month
```

### When to Upgrade
```
Free tier exhausted when:
- Processing >14,400 signals/day (you'll do ~250)
- Running 50+ cells (you'll run 5)
- High-frequency trading (>1 trade/minute)

Upgrade cost: ~$2-5/month (still very cheap!)
```

---

## Security Considerations

### API Keys
```bash
# Never commit API keys
echo "GROQ_API_KEY=*" >> .gitignore
echo ".env" >> .gitignore

# Use environment variables
export GROQ_API_KEY="..."
```

### Network Security
```bash
# Restrict Creature port to localhost only
# In creature/src/server.rs:
bind_address = "127.0.0.1:3030"  # Not 0.0.0.0

# Only Moon Dev (same VPS) can access
```

### Data Privacy
```bash
# Creature stores signal data locally
# Review data/thoughts/ and data/plans/ periodically
# Implement data retention policy (delete old files)
```

---

Ready to start building! Let me know if you want me to create the actual code files now.
