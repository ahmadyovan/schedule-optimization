use std::collections::HashMap;
use std::sync::{atomic::AtomicBool, Arc};
use serde_json::{json, Value};

use crate::algorithms::models::{
    CourseRequest, PSO, PsoParameters, TimePreferenceRequest,
};

pub struct ParamRange {
    pub swarm_size: (i32, i32),
    pub max_iterations: (usize, usize),
    pub inertia_weight: (f64, f64),
    pub cognitive_weight: (f64, f64),
    pub social_weight: (f64, f64),
}

pub async fn optimize_by_range(
    courses: &[CourseRequest],
    time_preferences: &[TimePreferenceRequest],
    param_range: ParamRange,
) -> (Value, HashMap<String, Vec<(PsoParameters, f64)>>) {
    println!("📊 Memulai optimasi PSO dengan parameter range:");
    println!("- swarm_size       : {:?}", param_range.swarm_size);
    println!("- max_iterations   : {:?}", param_range.max_iterations);
    println!("- inertia_weight   : {:?}", param_range.inertia_weight);
    println!("- cognitive_weight : {:?}", param_range.cognitive_weight);
    println!("- social_weight    : {:?}", param_range.social_weight);

    let mut full_experiments: HashMap<String, Vec<(PsoParameters, f64)>> = HashMap::new();

    let mut best_params = PsoParameters {
        swarm_size: param_range.swarm_size.0,
        max_iterations: param_range.max_iterations.0,
        inertia_weight: param_range.inertia_weight.0,
        cognitive_weight: param_range.cognitive_weight.0,
        social_weight: param_range.social_weight.0,
        num_runs: Some(1),
    };

    let mut history: HashMap<String, Vec<(f64, f64)>> = HashMap::new();

    fn range_float(start: f64, end: f64, step: f64) -> Vec<f64> {
        let mut result = Vec::new();
        let mut current = start;
        while current <= end {
            result.push(current);
            current += step;
        }
        result
    }

    fn range_int(start: i32, end: i32, step: i32) -> Vec<f64> {
        (start..=end).step_by(step as usize).map(|v| v as f64).collect()
    }

    async fn run_fitness(
        params: &PsoParameters,
        courses: &[CourseRequest],
        prefs: &[TimePreferenceRequest],
    ) -> f64 {
        println!(
            "⚙️  Menjalankan fitness dengan params: swarm={}, iter={}, iw={:.2}, cw={:.2}, sw={:.2}",
            params.swarm_size,
            params.max_iterations,
            params.inertia_weight,
            params.cognitive_weight,
            params.social_weight
        );

        let mut pso = PSO::new(courses.to_vec(), prefs.to_vec(), params.clone());
        let (_, fitness) = pso
            .optimize(None, None, &mut vec![], Arc::new(AtomicBool::new(false)))
            .await;

        println!("📈 Fitness: {:.4}", fitness);
        fitness
    }

    let steps = vec![
        ("swarm_size", range_int(param_range.swarm_size.0, param_range.swarm_size.1, 100)),
        (
            "max_iterations",
            range_int(
                param_range.max_iterations.0 as i32,
                param_range.max_iterations.1 as i32,
                100,
            ),
        ),
        (
            "inertia_weight",
            range_float(param_range.inertia_weight.0, param_range.inertia_weight.1, 0.1),
        ),
        (
            "cognitive_weight",
            range_float(param_range.cognitive_weight.0, param_range.cognitive_weight.1, 0.1),
        ),
        (
            "social_weight",
            range_float(param_range.social_weight.0, param_range.social_weight.1, 0.1),
        ),
    ];

    for (param_name, values) in steps {
        println!("\n🔧 Menyesuaikan parameter: {}", param_name);

        let mut best_val = values[0];
        let mut best_fitness = f64::INFINITY;
        let mut records = vec![];

        for (i, val) in values.iter().enumerate() {
            println!("🧪 [{}/{}] Menguji {} = {}", i + 1, values.len(), param_name, val);

            let mut test_params = best_params.clone();

            match param_name {
                "swarm_size" => test_params.swarm_size = *val as i32,
                "max_iterations" => test_params.max_iterations = *val as usize,
                "inertia_weight" => test_params.inertia_weight = *val,
                "cognitive_weight" => test_params.cognitive_weight = *val,
                "social_weight" => test_params.social_weight = *val,
                _ => {}
            }

            let fitness = run_fitness(&test_params, courses, time_preferences).await;
            records.push((*val, fitness));

            full_experiments.entry(param_name.to_string())
            .or_default()
            .push((test_params.clone(), fitness));


            if fitness < best_fitness {
                best_fitness = fitness;
                best_val = *val;
            }
        }

        match param_name {
            "swarm_size" => best_params.swarm_size = best_val as i32,
            "max_iterations" => best_params.max_iterations = best_val as usize,
            "inertia_weight" => best_params.inertia_weight = best_val,
            "cognitive_weight" => best_params.cognitive_weight = best_val,
            "social_weight" => best_params.social_weight = best_val,
            _ => {}
        }

        history.insert(param_name.to_string(), records);

        println!(
            "✅ Parameter {} terbaik: {} dengan fitness {:.4}",
            param_name, best_val, best_fitness
        );
    }

    println!("\n🚀 Menjalankan optimasi akhir dengan parameter terbaik...");

    let mut pso = PSO::new(courses.to_vec(), time_preferences.to_vec(), best_params.clone());
    let (_, fitness) = pso
        .optimize(None, None, &mut vec![], Arc::new(AtomicBool::new(false)))
        .await;

    println!("🏁 Optimasi selesai. Final Fitness: {:.4}", fitness);

    let json_result = json!({
        "fitness": fitness,
        "best_params": best_params,
        "experiments": history,
    });

    (json_result, full_experiments)
}
