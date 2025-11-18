// Trading Bridge API Server
// Exposes REST endpoints for Python Moon Dev agents to validate signals

use axum::{
    extract::{Json, State},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Instant;

mod groq_client;
use groq_client::{GroqClient, CellAnalysis};

#[derive(Debug, Deserialize)]
pub struct SignalRequest {
    signal_id: String,
    signal_type: String,
    timestamp: i64,
    data: serde_json::Value,
    context: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ValidationResponse {
    signal_id: String,
    validation_result: String,
    consensus_score: f64,
    confidence: f64,
    cell_votes: Vec<CellVoteResult>,
    aggregate_reasoning: String,
    recommended_action: String,
    suggested_position_size: f64,
    processing_time_ms: u128,
}

#[derive(Debug, Serialize, Clone)]
pub struct CellVoteResult {
    cell_id: u32,
    role: String,
    vote: String,
    confidence: f64,
    reasoning: String,
}

#[derive(Clone)]
struct AppState {
    groq_client: Arc<GroqClient>,
    colony_config: Arc<ColonyConfig>,
}

#[derive(Clone)]
struct ColonyConfig {
    cells: Vec<CellConfig>,
    consensus_threshold: f64,
}

#[derive(Clone)]
struct CellConfig {
    id: u32,
    role: String,
}

impl Default for ColonyConfig {
    fn default() -> Self {
        Self {
            cells: vec![
                CellConfig {
                    id: 1,
                    role: "historical_pattern_matcher".to_string(),
                },
                CellConfig {
                    id: 2,
                    role: "volume_analyzer".to_string(),
                },
                CellConfig {
                    id: 3,
                    role: "correlation_detector".to_string(),
                },
                CellConfig {
                    id: 4,
                    role: "risk_assessor".to_string(),
                },
                CellConfig {
                    id: 5,
                    role: "anomaly_detector".to_string(),
                },
            ],
            consensus_threshold: 0.7,
        }
    }
}

pub async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    let groq_client = GroqClient::new()?;

    let state = AppState {
        groq_client: Arc::new(groq_client),
        colony_config: Arc::new(ColonyConfig::default()),
    };

    let app = Router::new()
        .route("/api/v1/signal/validate", post(validate_signal))
        .route("/api/v1/colony/status", get(get_colony_status))
        .route("/health", get(health_check))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3030")
        .await
        .unwrap();

    println!("🚀 Trading Bridge Server running on http://127.0.0.1:3030");
    println!("📡 Endpoints:");
    println!("   POST /api/v1/signal/validate - Validate trading signal");
    println!("   GET  /api/v1/colony/status    - Get colony status");
    println!("   GET  /health                   - Health check");

    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn validate_signal(
    State(state): State<AppState>,
    Json(request): Json<SignalRequest>,
) -> impl IntoResponse {
    let start_time = Instant::now();

    println!("\n🔍 Validating signal: {}", request.signal_id);
    println!("   Type: {}", request.signal_type);

    // Format signal data for LLM analysis
    let signal_data = format!(
        "Signal Type: {}\nData: {}\nContext: {}",
        request.signal_type,
        serde_json::to_string_pretty(&request.data).unwrap_or_default(),
        serde_json::to_string_pretty(&request.context).unwrap_or_default()
    );

    // Get votes from all cells
    let mut cell_votes = Vec::new();

    for cell in &state.colony_config.cells {
        println!("   Querying Cell {} ({})...", cell.id, cell.role);

        match state
            .groq_client
            .analyze_signal(&signal_data, &cell.role)
            .await
        {
            Ok(analysis) => {
                let vote_result = CellVoteResult {
                    cell_id: cell.id,
                    role: cell.role.clone(),
                    vote: analysis.vote.clone(),
                    confidence: analysis.confidence,
                    reasoning: analysis.reasoning.clone(),
                };
                cell_votes.push(vote_result);
            }
            Err(e) => {
                eprintln!("   ❌ Cell {} failed: {}", cell.id, e);
                // Add fallback vote
                cell_votes.push(CellVoteResult {
                    cell_id: cell.id,
                    role: cell.role.clone(),
                    vote: "INVALID".to_string(),
                    confidence: 0.3,
                    reasoning: format!("Analysis failed: {}", e),
                });
            }
        }
    }

    // Calculate consensus
    let (consensus_score, avg_confidence, decision) =
        calculate_consensus(&cell_votes, state.colony_config.consensus_threshold);

    let recommended_action = if decision == "VALID" {
        "EXECUTE"
    } else {
        "REJECT"
    };

    let suggested_position_size = if decision == "VALID" {
        consensus_score.min(1.0)
    } else {
        0.0
    };

    // Generate aggregate reasoning
    let aggregate_reasoning = generate_aggregate_reasoning(&cell_votes, consensus_score);

    let processing_time_ms = start_time.elapsed().as_millis();

    println!("   ✅ Validation complete in {}ms", processing_time_ms);
    println!("   Decision: {}", decision);
    println!("   Consensus: {:.1}%", consensus_score * 100.0);

    let response = ValidationResponse {
        signal_id: request.signal_id,
        validation_result: decision,
        consensus_score,
        confidence: avg_confidence,
        cell_votes,
        aggregate_reasoning,
        recommended_action: recommended_action.to_string(),
        suggested_position_size,
        processing_time_ms,
    };

    Json(response)
}

fn calculate_consensus(votes: &[CellVoteResult], threshold: f64) -> (f64, f64, String) {
    let mut valid_score = 0.0;
    let mut invalid_score = 0.0;
    let mut total_confidence = 0.0;

    for vote in votes {
        total_confidence += vote.confidence;
        if vote.vote.to_uppercase() == "VALID" {
            valid_score += vote.confidence;
        } else {
            invalid_score += vote.confidence;
        }
    }

    let total_score = valid_score + invalid_score;
    let consensus_score = if total_score > 0.0 {
        valid_score / total_score
    } else {
        0.0
    };

    let avg_confidence = if !votes.is_empty() {
        total_confidence / votes.len() as f64
    } else {
        0.0
    };

    let decision = if consensus_score >= threshold {
        "VALID"
    } else {
        "INVALID"
    };

    (consensus_score, avg_confidence, decision.to_string())
}

fn generate_aggregate_reasoning(votes: &[CellVoteResult], consensus_score: f64) -> String {
    let valid_count = votes.iter().filter(|v| v.vote.to_uppercase() == "VALID").count();
    let total_count = votes.len();

    let mut reasoning = format!(
        "{}/{} cells recommend VALID (consensus: {:.1}%).\n\n",
        valid_count,
        total_count,
        consensus_score * 100.0
    );

    reasoning.push_str("Key insights:\n");
    for vote in votes {
        let emoji = if vote.vote.to_uppercase() == "VALID" {
            "✅"
        } else {
            "❌"
        };
        reasoning.push_str(&format!(
            "{} Cell {} ({}): {} ({:.0}% confidence)\n   {}\n",
            emoji,
            vote.cell_id,
            vote.role,
            vote.vote,
            vote.confidence * 100.0,
            vote.reasoning
        ));
    }

    reasoning
}

async fn get_colony_status(State(state): State<AppState>) -> impl IntoResponse {
    let status = serde_json::json!({
        "status": "healthy",
        "colony_size": state.colony_config.cells.len(),
        "consensus_threshold": state.colony_config.consensus_threshold,
        "cells": state.colony_config.cells.iter().map(|c| {
            serde_json::json!({
                "id": c.id,
                "role": c.role,
            })
        }).collect::<Vec<_>>(),
    });

    Json(status)
}

async fn health_check() -> &'static str {
    "OK"
}
