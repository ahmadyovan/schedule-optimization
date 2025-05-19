// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use serde_json::{json, Value};

mod algorithms;
use algorithms::optimizer::PSO;
use algorithms::models::{CourseRequest, TimePreferenceRequest, PsoParameters, OptimizedCourse};


#[derive(Clone, serde::Serialize)]
struct ProgressEvent {
    message: String,
    progress: f32,
    current_iteration: usize,
}

#[tauri::command]
async fn process_pso(
    course_csv: String,
    preference_csv: String,
    params: PsoParameters,
    window: tauri::Window,
) -> Result<Value, String> {
    let courses = parse_course_csv(&course_csv)?;
    let time_preferences = parse_preference_csv(&preference_csv)?;
    let num_runs = params.num_runs.unwrap_or(1);

    let mut best_overall_schedule: Option<Vec<OptimizedCourse>> = None;
    let mut best_overall_fitness = f32::INFINITY;
    let mut all_best_fitness = Vec::with_capacity(num_runs);

    for _ in 0..num_runs {
        let mut pso = PSO::new(
            courses.clone(),
            time_preferences.clone(),
            params.clone(),
        );

        let best_position = pso.optimize(&window, None).await;
        let schedule = PSO::position_to_schedule(&best_position, &courses);

        let (fitnes, conflicts) = pso.evaluate_best_position();
        all_best_fitness.push(fitnes);
        if fitnes < best_overall_fitness {
            best_overall_fitness = fitnes;
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
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![process_pso])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
