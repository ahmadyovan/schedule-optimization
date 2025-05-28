use core::num;
use std::time::Duration;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response, sse::{Event, Sse}},
    Json,
};
use futures::stream::Stream;
use serde_json::json;
use tokio::sync::watch;
use log::error;
use crate::models::{ConflictInfo, OptimizationRequest, OptimizationStatus};
use crate::pso::optimizer::PSO;

#[derive(Clone)]
pub struct AppState {
    pub status_tx: tokio::sync::broadcast::Sender<OptimizationStatus>,
    pub stop_tx: watch::Sender<bool>,
}

pub async fn stop_handler(
    State(state): State<AppState>,
) -> Result<Response, StatusCode> {
    // Hanya kirim sinyal stop sekali
    if state.stop_tx.send(true).is_err() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    Ok(Json(json!({ "success": true })).into_response())
}

pub async fn status_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>> + 'static> {
    let mut rx = state.status_tx.subscribe();
    
    let stream = async_stream::stream! {
        while let Ok(status) = rx.recv().await {
            match serde_json::to_string(&status) {
                Ok(data) => {
                    yield Ok(Event::default().data(data).event("status"));
                }
                Err(e) => error!("Serialization error: {}", e),
            }
        }
    };
    
    Sse::new(stream)
}

pub async fn optimize_handler(
    State(state): State<AppState>,
    Json(req): Json<OptimizationRequest>,
) -> Result<Response, StatusCode> {
    let courses = req.courses.clone();
    let time_preferences = req.time_preferences.clone();
    let sum_ruangan = req.sum_ruangan;
    let parameters = req.parameters.clone();
    let status_tx = state.status_tx.clone();
    let stop_rx = state.stop_tx.subscribe();
   

    // Ambil jumlah run dari request atau default 1 (single run)
    let num_runs = req.parameters.num_runs.unwrap_or(5);

    let result = tokio::task::spawn(async move {
        
        if num_runs == 1 {
            // Single run mode
            let mut pso = PSO::new(
                courses.clone(),
                time_preferences.clone(),
                parameters.clone(),
                sum_ruangan
            );
            
            let best_position = pso.optimize(status_tx.clone(), stop_rx.clone(), None).await;
            let (fitness, conflicts) = pso.evaluate_best_position();
            let schedule = PSO::position_to_schedule(&best_position, &courses);

            serde_json::json!({
                "success": true,
                "fitness": fitness,
                "conflicts": conflicts,
                "schedule": schedule
            })
        } else {
            // Multiple runs mode
            let mut all_best_fitness = Vec::with_capacity(num_runs);
            let mut best_overall_schedule = None;
            let mut best_overall_fitness = f32::INFINITY;
            let mut best_conflicts = ConflictInfo::default();

            for run in 0..num_runs {
                if *stop_rx.borrow() {
                    break;
                }

                let mut pso = PSO::new(
                    courses.clone(),
                    time_preferences.clone(),
                    parameters.clone(),
                    sum_ruangan
                );
                
                let best_position = pso.optimize(
                    status_tx.clone(), 
                    stop_rx.clone(), 
                    Some((run, num_runs))
                ).await;
                
                let (fitness, conflicts) = pso.evaluate_best_position();
                all_best_fitness.push(fitness);
                
                if fitness < best_overall_fitness {
                    best_overall_fitness = fitness;
                    best_conflicts = conflicts;
                    best_overall_schedule = Some(PSO::position_to_schedule(&best_position, &courses));
                }
            }
            
            serde_json::json!({
                "success": true,
                "fitness": best_overall_fitness,
                "conflicts": best_conflicts,
                "all_best_fitness": all_best_fitness,
                "schedule": best_overall_schedule
            })
        }
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(result).into_response())
}