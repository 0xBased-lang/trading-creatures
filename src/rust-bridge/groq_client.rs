// Groq API Client for Creature Framework
// Replaces expensive OpenRouter with free/cheap Groq API

use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::env;
use std::time::Duration;

#[derive(Debug, Serialize)]
struct GroqRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let api_key = env::var("GROQ_API_KEY")
            .map_err(|_| "GROQ_API_KEY environment variable not set")?;

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            api_key,
            model: "llama-3.1-70b-versatile".to_string(),
        })
    }

    pub async fn generate(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        let request = GroqRequest {
            model: self.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: 0.7,
            max_tokens: 1000,
        };

        let response = self
            .client
            .post("https://api.groq.com/openai/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(format!("Groq API error {}: {}", status, error_text).into());
        }

        let groq_response: GroqResponse = response.json().await?;

        if groq_response.choices.is_empty() {
            return Err("No response from Groq API".into());
        }

        Ok(groq_response.choices[0].message.content.clone())
    }

    pub async fn analyze_signal(
        &self,
        signal_data: &str,
        cell_role: &str,
    ) -> Result<CellAnalysis, Box<dyn std::error::Error>> {
        let prompt = self.build_analysis_prompt(signal_data, cell_role);
        let response = self.generate(&prompt).await?;
        self.parse_analysis(&response)
    }

    fn build_analysis_prompt(&self, signal_data: &str, cell_role: &str) -> String {
        match cell_role {
            "historical_pattern_matcher" => format!(
                "You are a historical pattern analysis expert analyzing trading signals.

SIGNAL DATA:
{}

TASK: Analyze if this signal matches successful historical patterns.
Consider wallet behavior, success rates, and market regime similarities.

Respond EXACTLY in this format:
VOTE: [VALID or INVALID]
CONFIDENCE: [number between 0.0 and 1.0]
REASONING: [Your detailed analysis in one paragraph]",
                signal_data
            ),
            "volume_analyzer" => format!(
                "You are a volume analysis expert analyzing trading signals.

SIGNAL DATA:
{}

TASK: Determine if volume profile supports this signal.
Consider volume/market-cap ratio, volume trends, and unusual spikes.

Respond EXACTLY in this format:
VOTE: [VALID or INVALID]
CONFIDENCE: [number between 0.0 and 1.0]
REASONING: [Your detailed analysis in one paragraph]",
                signal_data
            ),
            "correlation_detector" => format!(
                "You are a correlation analysis expert analyzing trading signals.

SIGNAL DATA:
{}

TASK: Analyze correlations with other assets and market movements.
Consider if this signal aligns with broader market trends.

Respond EXACTLY in this format:
VOTE: [VALID or INVALID]
CONFIDENCE: [number between 0.0 and 1.0]
REASONING: [Your detailed analysis in one paragraph]",
                signal_data
            ),
            "risk_assessor" => format!(
                "You are a risk management expert analyzing trading signals.

SIGNAL DATA:
{}

TASK: Assess risk/reward ratio and potential downsides.
Consider liquidation risk, position sizing, and market volatility.

Respond EXACTLY in this format:
VOTE: [VALID or INVALID]
CONFIDENCE: [number between 0.0 and 1.0]
REASONING: [Your detailed analysis in one paragraph]",
                signal_data
            ),
            "anomaly_detector" => format!(
                "You are an anomaly detection expert analyzing trading signals.

SIGNAL DATA:
{}

TASK: Identify if this signal shows unusual patterns or potential manipulation.
Look for wash trading, spoofing, or other suspicious activity.

Respond EXACTLY in this format:
VOTE: [VALID or INVALID]
CONFIDENCE: [number between 0.0 and 1.0]
REASONING: [Your detailed analysis in one paragraph]",
                signal_data
            ),
            _ => format!(
                "Analyze this trading signal:

{}

Respond with:
VOTE: [VALID or INVALID]
CONFIDENCE: [0.0-1.0]
REASONING: [Your analysis]",
                signal_data
            ),
        }
    }

    fn parse_analysis(&self, response: &str) -> Result<CellAnalysis, Box<dyn std::error::Error>> {
        let mut vote = "INVALID";
        let mut confidence = 0.5;
        let mut reasoning = String::new();

        for line in response.lines() {
            if line.starts_with("VOTE:") {
                vote = line.trim_start_matches("VOTE:").trim();
            } else if line.starts_with("CONFIDENCE:") {
                let conf_str = line.trim_start_matches("CONFIDENCE:").trim();
                confidence = conf_str.parse().unwrap_or(0.5);
            } else if line.starts_with("REASONING:") {
                reasoning = line.trim_start_matches("REASONING:").trim().to_string();
            }
        }

        Ok(CellAnalysis {
            vote: vote.to_string(),
            confidence,
            reasoning,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CellAnalysis {
    pub vote: String,
    pub confidence: f64,
    pub reasoning: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_groq_client() {
        // Only run if GROQ_API_KEY is set
        if env::var("GROQ_API_KEY").is_ok() {
            let client = GroqClient::new().unwrap();
            let result = client.generate("Say 'hello' in one word").await;
            assert!(result.is_ok());
            println!("Groq response: {:?}", result);
        }
    }
}
