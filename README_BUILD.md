# BUILD AND RUN - Trading Creatures

**Stop reading, start building!** Here's how to get this running on your VPS.

---

## 🚀 Quick Deploy (30 minutes)

### Step 1: Get Groq API Key (5 min)
```bash
# 1. Go to https://console.groq.com/keys
# 2. Sign up (free)
# 3. Create API key
# 4. Copy it

export GROQ_API_KEY="gsk_your_key_here"
echo 'export GROQ_API_KEY="gsk_your_key_here"' >> ~/.bashrc
```

### Step 2: Install Dependencies (10 min)
```bash
# Install Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Verify
rustc --version
```

### Step 3: Build and Run (15 min)
```bash
cd ~/trading-creatures

# Build the Rust bridge
cd src/rust-bridge
cargo build --release

# Run it!
./scripts/run_bridge.sh
```

Server should start on `http://localhost:3030`

---

## ✅ Test It Works

Open **another terminal** and run:

```bash
cd ~/trading-creatures

# Install Python dependencies
pip3 install aiohttp

# Run test suite
python3 scripts/test_bridge.py
```

If you see:
```
✅ Connection Test PASSED
✅ Colony Status PASSED
✅ Whale Signal Validation PASSED
✅ Multiple Signals PASSED
```

**You're done!** System is working.

---

## 🔌 Integrate with Moon Dev

Now add validation to your whale agent:

```bash
# Copy creature_client.py to Moon Dev
cp src/python-client/creature_client.py ~/moon-dev-ai-agents/src/utils/

# Add to your whale_agent.py:
```

```python
from utils.creature_client import CreatureClient

async def process_whale_signal(wallet, amount, token):
    # Your existing detection logic...

    # NEW: Validate with Creature
    async with CreatureClient() as client:
        result = await client.validate_signal(
            "whale",
            {
                "wallet_address": wallet,
                "amount_usd": amount,
                "token": token,
                "current_price": price,
                "volume_24h": volume
            }
        )

    # Only trade if validated
    if result.recommended_action == "EXECUTE":
        execute_trade(token, amount * result.suggested_position_size)
        print(f"✅ Trade executed: {result.aggregate_reasoning}")
    else:
        print(f"🚫 Trade rejected: {result.aggregate_reasoning}")
```

---

## 📊 What You Built

✅ **Rust server** with Groq API (5 specialized AI cells)
✅ **REST API** for signal validation (`/api/v1/signal/validate`)
✅ **Python client** for Moon Dev integration
✅ **Multi-cell consensus** validation system

**Cost:** $0/month (Groq free tier)
**Latency:** <500ms per validation
**Accuracy:** 5 AI cells vote on every signal

---

## 🐛 Troubleshooting

### "GROQ_API_KEY not set"
```bash
export GROQ_API_KEY="your-key"
```

### "Port 3030 already in use"
```bash
# Kill existing process
lsof -i :3030
kill -9 <PID>
```

### "Cargo command not found"
```bash
source $HOME/.cargo/env
```

### Python tests fail
```bash
pip3 install aiohttp
```

---

## Next Steps

1. **Run for 1 week** - Collect data on signal validation
2. **Measure improvement** - Compare win rate before/after
3. **Tune thresholds** - Adjust consensus threshold in `config/`
4. **Add more signals** - Expand beyond whale to sentiment, volume, etc.

Questions? Check the detailed docs or ask!
