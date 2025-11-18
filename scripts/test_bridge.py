#!/usr/bin/env python3
"""
Test script for Trading Bridge
Validates that Creature colony can validate signals
"""

import asyncio
import sys
import os

# Add parent directory to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'src', 'python-client'))

from creature_client import CreatureClient


async def test_basic_connection():
    """Test 1: Basic connection to Creature server"""
    print("\n" + "="*60)
    print("TEST 1: Basic Connection")
    print("="*60)

    async with CreatureClient() as client:
        is_healthy = await client.health_check()
        if is_healthy:
            print("✅ Server is responding")
        else:
            print("❌ Server not responding")
            return False

    return True


async def test_colony_status():
    """Test 2: Get colony status"""
    print("\n" + "="*60)
    print("TEST 2: Colony Status")
    print("="*60)

    async with CreatureClient() as client:
        status = await client.get_colony_status()
        print(f"\n📊 Colony Status:")
        print(f"   Status: {status.get('status')}")
        print(f"   Colony Size: {status.get('colony_size')} cells")
        print(f"   Consensus Threshold: {status.get('consensus_threshold')}")

        print(f"\n🔬 Cells:")
        for cell in status.get('cells', []):
            print(f"   Cell {cell['id']}: {cell['role']}")

    return True


async def test_whale_signal_validation():
    """Test 3: Validate whale signal"""
    print("\n" + "="*60)
    print("TEST 3: Whale Signal Validation")
    print("="*60)

    whale_signal = {
        "wallet_address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
        "token": "BTC",
        "action": "buy",
        "amount_usd": 500000,
        "current_price": 43250.50,
        "volume_24h": 28500000000,
    }

    context = {
        "market_regime": "ranging",
        "sentiment_score": 0.65,
        "funding_rate": 0.0125
    }

    print("\n🐋 Submitting whale signal for validation...")
    print(f"   Wallet: {whale_signal['wallet_address'][:10]}...")
    print(f"   Amount: ${whale_signal['amount_usd']:,.0f}")
    print(f"   Token: {whale_signal['token']}")

    async with CreatureClient() as client:
        result = await client.validate_signal("whale", whale_signal, context)

        print(f"\n📊 Validation Results:")
        print(f"   Signal ID: {result.signal_id}")
        print(f"   Decision: {result.decision}")
        print(f"   Consensus Score: {result.consensus_score:.1%}")
        print(f"   Confidence: {result.confidence:.1%}")
        print(f"   Recommended Action: {result.recommended_action}")
        print(f"   Position Size: {result.suggested_position_size:.1%}")
        print(f"   Processing Time: {result.processing_time_ms}ms")

        print(f"\n🔬 Cell Votes:")
        for vote in result.cell_votes:
            emoji = "✅" if vote.vote == "VALID" else "❌"
            print(f"   {emoji} Cell {vote.cell_id} ({vote.role})")
            print(f"      Vote: {vote.vote}")
            print(f"      Confidence: {vote.confidence:.1%}")
            print(f"      Reasoning: {vote.reasoning[:100]}...")

        print(f"\n💭 Aggregate Reasoning:")
        print(f"   {result.aggregate_reasoning[:500]}...")

        # Final decision
        if result.recommended_action == "EXECUTE":
            print(f"\n✅ TRADE APPROVED - Consensus {result.consensus_score:.1%}")
        else:
            print(f"\n🚫 TRADE REJECTED - Consensus {result.consensus_score:.1%}")

    return True


async def test_multiple_signals():
    """Test 4: Validate multiple signals rapidly"""
    print("\n" + "="*60)
    print("TEST 4: Multiple Signal Validation (Throughput Test)")
    print("="*60)

    signals = [
        {
            "type": "whale",
            "data": {"wallet": "0x123", "amount_usd": 100000, "token": "ETH"}
        },
        {
            "type": "whale",
            "data": {"wallet": "0x456", "amount_usd": 250000, "token": "SOL"}
        },
        {
            "type": "whale",
            "data": {"wallet": "0x789", "amount_usd": 500000, "token": "BTC"}
        }
    ]

    async with CreatureClient() as client:
        import time
        start_time = time.time()

        tasks = [
            client.validate_signal(sig["type"], sig["data"])
            for sig in signals
        ]

        results = await asyncio.gather(*tasks)

        elapsed = time.time() - start_time

        print(f"\n✅ Validated {len(results)} signals in {elapsed:.2f}s")
        print(f"   Average: {elapsed/len(results):.2f}s per signal")

        for i, result in enumerate(results):
            print(f"\n   Signal {i+1}: {result.decision} ({result.consensus_score:.0%} consensus)")

    return True


async def main():
    """Run all tests"""
    print("\n🧬 Trading Creatures Bridge - Test Suite")
    print("="*60)

    tests = [
        ("Connection Test", test_basic_connection),
        ("Colony Status", test_colony_status),
        ("Whale Signal Validation", test_whale_signal_validation),
        ("Multiple Signals", test_multiple_signals),
    ]

    passed = 0
    failed = 0

    for name, test_func in tests:
        try:
            result = await test_func()
            if result:
                passed += 1
                print(f"\n✅ {name} PASSED")
            else:
                failed += 1
                print(f"\n❌ {name} FAILED")
        except Exception as e:
            failed += 1
            print(f"\n❌ {name} FAILED with error: {e}")

    print("\n" + "="*60)
    print(f"TEST RESULTS: {passed} passed, {failed} failed")
    print("="*60)

    return failed == 0


if __name__ == "__main__":
    success = asyncio.run(main())
    sys.exit(0 if success else 1)
