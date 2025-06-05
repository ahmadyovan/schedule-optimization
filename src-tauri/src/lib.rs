// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use serde_json::{json, Value};

mod algorithms;
use algorithms::models::{ CourseRequest, OptimizedCourse, PSO, PsoParameters, TimePreferenceRequest};

use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use tauri::State;

#[derive(Default)]
pub struct AppState {
    pub stop_flag: Mutex<Option<Arc<AtomicBool>>>,
}

#[tauri::command]
fn stop_pso(state: State<'_, AppState>) {
    if let Some(flag) = &*state.stop_flag.lock().unwrap() {
        flag.store(true, Ordering::Relaxed);
    }
}

#[tauri::command]
async fn process_pso(
    course_csv: String,
    preference_csv: String,
    params: PsoParameters,
    window: tauri::Window,
    state: State<'_, AppState>, // Tambahan
) -> Result<Value, String> {
    let courses = parse_course_csv(&course_csv)?;
    let time_preferences = parse_preference_csv(&preference_csv)?;
    let num_runs = params.num_runs.unwrap_or(1);

    let stop_flag = Arc::new(AtomicBool::new(false));
    {
        let mut flag = state.stop_flag.lock().unwrap();
        *flag = Some(stop_flag.clone());
    }

    let mut best_overall_schedule: Option<Vec<OptimizedCourse>> = None;
    let mut best_overall_fitness = f32::INFINITY;
    let mut all_best_fitness = Vec::with_capacity(num_runs);

    for i in 0..num_runs {
        let mut pso = PSO::new(
            courses.clone(),
            time_preferences.clone(),
            params.clone(),
        );

        let (best_position, fitness) =
            pso.optimize(&window, Some((i, num_runs)), &mut all_best_fitness, stop_flag.clone()).await;

        if stop_flag.load(Ordering::Relaxed) {
            break; // keluar dari loop jika dihentikan
        }

        let schedule = PSO::position_to_schedule(&best_position, &courses);

        if fitness < best_overall_fitness {
            best_overall_fitness = fitness;
            best_overall_schedule = Some(schedule);
        }
    }

    let result = json!({
        "success": true,
        "fitness": best_overall_fitness,
        "all_best_fitness": all_best_fitness,
        "schedule": best_overall_schedule
    });

    Ok(result)
}

// Helper functions for parsing
fn parse_course_csv(csv: &str) -> Result<Vec<CourseRequest>, String> {
    let mut rdr = csv::Reader::from_reader(csv.as_bytes());
    rdr.deserialize()
        .map(|result| result.map_err(|e| format!("CSV parse error: {}", e)))
        .collect()
}

fn parse_preference_csv(csv: &str) -> Result<Vec<TimePreferenceRequest>, String> {
    let mut rdr = csv::Reader::from_reader(csv.as_bytes());
    rdr.deserialize()
        .map(|result| result.map_err(|e| format!("Preference CSV error: {}", e)))
        .collect()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![process_pso, stop_pso])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
