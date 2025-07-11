use schedule_optimization_lib::algorithms::models::{
    CourseRequest, PsoParameters, TimePreferenceRequest
};

use indicatif::{ProgressBar, ProgressStyle};
use std::time::Instant;

use schedule_optimization_lib::algorithms::tune::{optimize_by_range, ParamRange};

use std::fs;
use tokio::runtime::Runtime;

use rust_xlsxwriter::{Workbook, Format, XlsxError};
use std::collections::HashMap;

pub fn export_full_experiments_single_sheet(
    data: &HashMap<String, Vec<(PsoParameters, f64)>>,
    filename: &str,
) -> Result<(), XlsxError> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    let bold = Format::new().set_bold();

    let headers = [
        "swarm_size",
        "max_iterations",
        "inertia_weight",
        "cognitive_weight",
        "social_weight",
        "fitness",
    ];

    let mut current_row = 0;

    for (param_name, trials) in data {
        // === Judul Subtabel ===
        let title = format!("=== Pengujian {} ===", param_name.replace('_', " ").to_uppercase());
        worksheet.write_with_format(current_row, 0, &title, &bold)?;
        current_row += 1;

        // Header Tabel
        for (col, title) in headers.iter().enumerate() {
            worksheet.write_with_format(current_row, col as u16, *title, &bold)?;
        }
        current_row += 1;

        // Data
        for (params, fitness) in trials {
            worksheet.write(current_row, 0, params.swarm_size)?;
            worksheet.write(current_row, 1, params.max_iterations as i32)?;
            worksheet.write(current_row, 2, params.inertia_weight)?;
            worksheet.write(current_row, 3, params.cognitive_weight)?;
            worksheet.write(current_row, 4, params.social_weight)?;
            worksheet.write(current_row, 5, *fitness)?;
            current_row += 1;
        }

        current_row += 2; // Spacer antar sub-tabel
    }

    workbook.save(filename)?;
    Ok(())
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

fn main() {
    println!("📥 Membaca file CSV...");
    let course_csv = fs::read_to_string("data/course.csv").expect("Gagal membaca file course.csv");
    let preference_csv = fs::read_to_string("data/preference.csv").expect("Gagal membaca file preference.csv");

    println!("✅ File CSV berhasil dibaca.");

    let courses = parse_course_csv(&course_csv).expect("Gagal parse course.csv");
    let prefs = parse_preference_csv(&preference_csv).expect("Gagal parse preference.csv");

    println!(
        "✅ Data berhasil di-parse. Jumlah course: {}, prefs: {}",
        courses.len(),
        prefs.len()
    );

    let rt = Runtime::new().expect("Gagal membuat Tokio runtime");

    println!("🚀 Mulai proses optimasi PSO...");

    // Mulai hitung waktu total
    let start_time = Instant::now();

    // Progress bar dummy (nanti bisa kamu hubungkan ke langkah internal kalau diinginkan)
    let pb = ProgressBar::new_spinner();
    pb.set_message("Mengoptimasi parameter...");
    pb.enable_steady_tick(std::time::Duration::from_millis(120));
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ")
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );

    let (result, full_experiments) = rt.block_on(optimize_by_range(
        &courses,
        &prefs,
        ParamRange {
            swarm_size: (100, 500),
            max_iterations: (500, 1000),
            inertia_weight: (0.5, 0.9),
            cognitive_weight: (1.0, 3.0),
            social_weight: (1.0, 3.0),
        },
    ));

    pb.finish_with_message("✅ Proses optimasi selesai.");

    // Hitung waktu selesai
    let elapsed = start_time.elapsed();
    let minutes = elapsed.as_secs() / 60;
    let seconds = elapsed.as_secs() % 60;
    println!("🕒 Waktu total: {} menit {} detik", minutes, seconds);

    // Cetak hasil dari JSON
    println!("=== Hasil Optimasi PSO ===");
    println!(
        "\nFitness: {:.2}",
        result["fitness"].as_f64().unwrap_or_default()
    );

    println!("\n=== Parameter Terbaik ===");
    if let Some(params) = result.get("best_params") {
        println!("Swarm Size     : {}", params["swarm_size"]);
        println!("Max Iterations : {}", params["max_iterations"]);
        println!("Inertia Weight : {}", params["inertia_weight"]);
        println!("Cognitive W    : {}", params["cognitive_weight"]);
        println!("Social W       : {}", params["social_weight"]);
    }

    export_full_experiments_single_sheet(&full_experiments, "pengujian_pso.xlsx").unwrap();
}
