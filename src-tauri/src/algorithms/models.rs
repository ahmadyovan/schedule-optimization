use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};

#[derive(Debug, Clone)]
pub struct Particle {
    pub position: Vec<f64>,
    pub velocity: Vec<f64>,
    pub pbest_position: Vec<f64>,
    pub pbest_fitness: f64,
    pub fitness: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CourseRequest {
    #[serde(rename = "id")] 
    pub id_jadwal: u32,
    pub id_matkul: u32,
    pub id_dosen: u32,
    pub id_waktu: u32,
    pub id_kelas: u32,
    pub semester: u32,
    pub sks: u32,
    pub prodi: u32,
}

#[derive(Clone, Serialize)]
pub struct Status {
    pub message: String
}

#[derive(Clone, serde::Serialize)]
pub struct OptimizationProgress {
     pub iteration: usize,
        pub elapsed_time: Duration,
        pub best_fitness: f64,
        pub all_best_fitness: Option<Vec<f64>>,  // Menjadi opsional
        pub current_run: Option<usize>,          // Menjadi opsional
        pub total_runs: Option<usize>,           // Menjadi opsional
        pub is_finished: bool,
        // pub conflicts: ConflictInfo,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TimePreferenceRequest {
    pub id_dosen: u32,
    pub senin_pagi: bool,
    pub senin_malam: bool,
    pub selasa_pagi: bool,
    pub selasa_malam: bool,
    pub rabu_pagi: bool,
    pub rabu_malam: bool,
    pub kamis_pagi: bool,
    pub kamis_malam: bool,
    pub jumat_pagi: bool,
    pub jumat_malam: bool,
}

#[derive(Debug, Clone)]
pub struct FitnessCalculator {
    pub time_preferences: HashMap<u32, TimePreferenceRequest>,
}

#[derive(Debug, Serialize, Clone)]
pub struct OptimizedCourse {
    pub id_jadwal: u32,
    pub id_matkul: u32,
    pub id_dosen: u32,
    pub id_kelas: u32,
    pub id_waktu: u32,
    pub hari: u32,
    pub jam_mulai: u32,
    pub jam_akhir: u32,
    pub ruangan: u32,
    pub semester: u32,
    pub sks: u32,
    pub prodi: u32,
}

pub struct PSO {
    pub particles: Vec<Particle>,
    pub global_best_position: Vec<f64>,
    pub global_best_fitness: f64,
    pub parameters: PsoParameters,
    pub courses: Vec<CourseRequest>,
    pub checker: ScheduleChecker,
}

#[derive(Debug, Clone)]
pub struct ScheduleChecker {
   pub time_preferences: HashMap<u32, TimePreferenceRequest>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PsoParameters {
    pub swarm_size: i32,
    pub max_iterations: usize,
    pub cognitive_weight: f64,
    pub social_weight: f64,
    pub inertia_weight: f64,
    pub num_runs: Option<usize>
}


#[derive(Debug, Clone, Serialize, Default)]
pub struct ConflictInfo {
    pub group_conflicts: Vec<((u32, u32, u32), (u32, u32, u32))>,
    pub preference_conflicts: Vec<u32>,
    pub conflicts_list: Vec<String>,
    pub total_conflicts: u32,
}

#[derive(Debug, Serialize, Clone)]
pub struct OptimizationStatus {
    pub iteration: usize,
    pub elapsed_time: Duration,
    pub current_fitness: f64,
    pub best_fitness: f64,
    pub all_best_fitness: Option<Vec<f64>>,  // Menjadi opsional
    pub current_run: Option<usize>,          // Menjadi opsional
    pub total_runs: Option<usize>,           // Menjadi opsional
    pub is_finished: bool,
    pub conflicts: ConflictInfo,
}