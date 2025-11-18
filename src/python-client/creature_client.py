"""
Creature Client for Moon Dev AI Agents
Async Python client for validating trading signals with Creature colony
"""

import aiohttp
import asyncio
import time
import json
from typing import Dict, List, Optional
from dataclasses import dataclass, asdict
from enum import Enum


class ValidationDecision(Enum):
    VALID = "VALID"
    INVALID = "INVALID"
    UNCERTAIN = "UNCERTAIN"


@dataclass
class CellVote:
    cell_id: int
    role: str
    vote: str
    confidence: float
    reasoning: str


@dataclass
class ValidationResult:
    signal_id: str
    decision: str
    consensus_score: float
    confidence: float
    cell_votes: List[CellVote]
    aggregate_reasoning: str
    recommended_action: str
    suggested_position_size: float
    processing_time_ms: int

    def to_dict(self) -> Dict:
        return {
            **asdict(self),
            'cell_votes': [asdict(v) for v in self.cell_votes]
        }


class CreatureClient:
    """
    Async client for communicating with Creature signal validation colony

    Usage:
        async with CreatureClient() as client:
            result = await client.validate_signal("whale", signal_data)
            if result.decision == "VALID":
                execute_trade()
    """

    def __init__(
        self,
        host: str = "localhost",
        port: int = 3030,
        timeout: float = 10.0,
        fallback_mode: str = "conservative"
    ):
        self.base_url = f"http://{host}:{port}"
        self.timeout = aiohttp.ClientTimeout(total=timeout)
        self.fallback_mode = fallback_mode
        self.session: Optional[aiohttp.ClientSession] = None
        self._request_count = 0

    async def __aenter__(self):
        self.session = aiohttp.ClientSession(timeout=self.timeout)
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self.session:
            await self.session.close()

    async def validate_signal(
        self,
        signal_type: str,
        data: Dict,
        context: Optional[Dict] = None
    ) -> ValidationResult:
        """
        Submit signal for validation by Creature colony

        Args:
            signal_type: Type of signal (whale, sentiment, volume, liquidation)
            data: Signal data (wallet, amount, token, price, etc.)
            context: Additional market context (optional)

        Returns:
            ValidationResult with colony consensus and recommendations
        """
        if self.session is None:
            raise RuntimeError("Client not initialized. Use 'async with CreatureClient()' pattern")

        signal_id = self._generate_signal_id(signal_type)

        signal_payload = {
            "signal_id": signal_id,
            "signal_type": signal_type,
            "timestamp": int(time.time()),
            "data": data,
            "context": context or {}
        }

        try:
            async with self.session.post(
                f"{self.base_url}/api/v1/signal/validate",
                json=signal_payload
            ) as response:
                self._request_count += 1

                if response.status == 200:
                    result = await response.json()
                    return self._parse_validation_result(result)
                else:
                    error_text = await response.text()
                    print(f"⚠️  Creature API error {response.status}: {error_text}")
                    return self._fallback_validation(signal_payload)

        except aiohttp.ClientError as e:
            print(f"⚠️  Creature connection failed: {e}")
            return self._fallback_validation(signal_payload)
        except Exception as e:
            print(f"⚠️  Unexpected error: {e}")
            return self._fallback_validation(signal_payload)

    async def get_colony_status(self) -> Dict:
        """Get current colony health and configuration"""
        if self.session is None:
            raise RuntimeError("Client not initialized")

        try:
            async with self.session.get(
                f"{self.base_url}/api/v1/colony/status"
            ) as response:
                return await response.json()
        except Exception as e:
            return {"error": str(e), "status": "unavailable"}

    async def health_check(self) -> bool:
        """Check if Creature server is responding"""
        if self.session is None:
            raise RuntimeError("Client not initialized")

        try:
            async with self.session.get(
                f"{self.base_url}/health"
            ) as response:
                return response.status == 200
        except:
            return False

    def _generate_signal_id(self, signal_type: str) -> str:
        import random
        timestamp = int(time.time())
        random_suffix = random.randint(1000, 9999)
        return f"{signal_type}_{timestamp}_{random_suffix}"

    def _parse_validation_result(self, data: Dict) -> ValidationResult:
        cell_votes = [
            CellVote(**vote) for vote in data.get("cell_votes", [])
        ]

        return ValidationResult(
            signal_id=data["signal_id"],
            decision=data["validation_result"],
            consensus_score=data["consensus_score"],
            confidence=data["confidence"],
            cell_votes=cell_votes,
            aggregate_reasoning=data["aggregate_reasoning"],
            recommended_action=data["recommended_action"],
            suggested_position_size=data.get("suggested_position_size", 0.0),
            processing_time_ms=data.get("processing_time_ms", 0)
        )

    def _fallback_validation(self, signal: Dict) -> ValidationResult:
        """
        Fallback logic when Creature is unavailable

        Modes:
        - conservative: Reject all signals (safe default)
        - aggressive: Accept all signals (risky)
        - disable: Pass through without validation
        """
        if self.fallback_mode == "conservative":
            decision = "INVALID"
            reasoning = "⚠️  Creature unavailable - conservative mode rejects signal for safety"
            action = "REJECT"
            position_size = 0.0
        elif self.fallback_mode == "aggressive":
            decision = "VALID"
            reasoning = "⚠️  Creature unavailable - aggressive mode accepts signal (USE WITH CAUTION)"
            action = "EXECUTE"
            position_size = 1.0
        else:  # disable
            decision = "VALID"
            reasoning = "⚠️  Creature unavailable - validation bypassed"
            action = "EXECUTE"
            position_size = 1.0

        return ValidationResult(
            signal_id=signal["signal_id"],
            decision=decision,
            consensus_score=0.5,
            confidence=0.0,
            cell_votes=[],
            aggregate_reasoning=reasoning,
            recommended_action=action,
            suggested_position_size=position_size,
            processing_time_ms=0
        )

    def get_stats(self) -> Dict:
        """Get client usage statistics"""
        return {
            "total_requests": self._request_count,
            "base_url": self.base_url,
            "fallback_mode": self.fallback_mode
        }


# Example usage
async def example_usage():
    """Example: How to use CreatureClient in your trading agents"""

    # Initialize client
    async with CreatureClient(fallback_mode="conservative") as client:

        # Check if Creature is available
        if await client.health_check():
            print("✅ Creature colony is healthy")
        else:
            print("❌ Creature colony unavailable")

        # Example whale signal
        whale_signal = {
            "wallet_address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
            "token": "BTC",
            "action": "buy",
            "amount_usd": 500000,
            "current_price": 43250.50,
            "volume_24h": 28500000000
        }

        context = {
            "market_regime": "ranging",
            "sentiment_score": 0.65,
            "funding_rate": 0.0125
        }

        # Validate signal
        print("\n🔍 Validating whale signal...")
        result = await client.validate_signal("whale", whale_signal, context)

        # Display results
        print(f"\n📊 Validation Results:")
        print(f"   Decision: {result.decision}")
        print(f"   Consensus: {result.consensus_score:.1%}")
        print(f"   Confidence: {result.confidence:.1%}")
        print(f"   Action: {result.recommended_action}")
        print(f"   Position Size: {result.suggested_position_size:.1%}")
        print(f"   Processing Time: {result.processing_time_ms}ms")

        print(f"\n💭 Colony Reasoning:")
        print(result.aggregate_reasoning)

        # Make trading decision
        if result.recommended_action == "EXECUTE" and result.consensus_score > 0.7:
            print("\n✅ Signal VALIDATED - Safe to execute trade")
        else:
            print("\n🚫 Signal REJECTED - Do not execute trade")

        # Get stats
        stats = client.get_stats()
        print(f"\n📈 Client Stats: {stats}")


if __name__ == "__main__":
    # Run example
    asyncio.run(example_usage())
