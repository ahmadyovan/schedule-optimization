use std::{
    sync::{Arc, atomic::{AtomicBool, Ordering}},
    time::Instant,
    collections::HashMap,
};

use rand::Rng;
use rayon::prelude::*;
use tauri::{Emitter, Window};

use super::{models::{
        CourseRequest, OptimizationProgress, OptimizedCourse, Particle, PsoParameters, ScheduleChecker, TimePreferenceRequest, PSO
}};

// ============================================================================
// PARTICLE IMPLEMENTATION
// ============================================================================
impl Particle {
    /// Create new particle with random position and velocity
    pub fn new(dimension: usize) -> Self {
        let mut rng = rand::rng();
        
        // Random position in [0,1] range
        let position: Vec<f64> = (0..dimension)
            .map(|_| rng.random_range(0.0..1.0))
            .collect();
            
        // Small random velocity for stable convergence
        let velocity: Vec<f64> = (0..dimension)
            .map(|_| rng.random_range(-0.1..0.1))
            .collect();

        Particle {
            position,
            velocity,
            pbest_position: vec![0.0; dimension], // Will be set after first evaluation
            pbest_fitness: f64::INFINITY,        // Initialize with infinity
            fitness: f64::INFINITY,              // Will be calculated in first iteration
        }
    }

    /// Update velocity using standard PSO formula
    pub fn update_velocity(
        &mut self,
        gbest: &[f64],
        inertia_weight: f64,
        cognitive_weight: f64,
        social_weight: f64,
    ) {
        let mut rng = rand::rng();
        
        for i in 0..self.velocity.len() {
            let r1: f64 = rng.random(); 
            let r2: f64 = rng.random(); 
            
            let cognitive = cognitive_weight * r1 * (self.pbest_position[i] - self.position[i]);
            
            let social = social_weight * r2 * (gbest[i] - self.position[i]);
            
            self.velocity[i] = inertia_weight * self.velocity[i] + cognitive + social;
            
        }
    }

    pub fn update_position(&mut self) {
        for i in 0..self.position.len() {
            self.position[i] += self.velocity[i];

        }
    }

    pub fn update_personal_best(&mut self) {
        if self.fitness < self.pbest_fitness && !self.fitness.is_nan() {
            self.pbest_fitness = self.fitness;
            self.pbest_position = self.position.clone();
        }
    }
}

// ============================================================================
// PSO IMPLEMENTATION
// ============================================================================
impl PSO {
    /// Constructor for new PSO instance
    pub fn new(
        courses: Vec<CourseRequest>,
        time_preferences: Vec<TimePreferenceRequest>,
        parameters: PsoParameters,
    ) -> Self {
        let dimension = courses.len() * 2; // 2 dimensions per course: day_order, time_order

        PSO {
            particles: vec![],
            global_best_position: vec![0.0; dimension],
            global_best_fitness: f64::INFINITY,
            courses,
            parameters,
            checker: ScheduleChecker::new(time_preferences),
        }
    }

    /// Main PSO optimization function
    pub async fn optimize(
        &mut self,
        window: Option<&Window>,
        run_info: Option<(usize, usize)>,
        all_best_fitness: &mut Vec<f64>,
        stop_flag: Arc<AtomicBool>,
    ) -> (Vec<f64>, f64) {
        let start_time = Instant::now();
        let (current_run, total_runs) = run_info.unwrap_or((0, 0));

        // Reset state for new run
        self.reset_optimization();

        // Initialize swarm with random particles
        self.initialize_swarm();

        // Main optimization loop
        for iteration in 0..self.parameters.max_iterations {
            
            if stop_flag.load(Ordering::Relaxed) {
                break;
            }

            // Step 1: Evaluate all particles
            self.evaluate_all_particles();

            // Step 2: Update global best
            self.update_global_best();

            // Step 3: Update all particles (velocity and position)
            self.update_all_particles();

            // Progress reporting
            if let Some(w) = window {
                self.emit_progress(Some(w), iteration + 1, &start_time, all_best_fitness, current_run, total_runs, false);
            }

            // Early stopping for very good solutions
            if self.global_best_fitness < 0.001 {
                break;
            }

           
        }

        // Final results
        all_best_fitness.push(self.global_best_fitness);
        self.emit_progress(window, self.parameters.max_iterations, &start_time, all_best_fitness, current_run, total_runs, true);

        (self.global_best_position.clone(), self.global_best_fitness)
    }

    /// Reset optimization state for new run
    fn reset_optimization(&mut self) {
        self.global_best_fitness = f64::INFINITY;
        self.global_best_position.fill(0.0);
        self.particles.clear();
    }

    /// Initialize swarm with random particles (no fitness evaluation here)
    fn initialize_swarm(&mut self) {
        let dimension = self.courses.len() * 2;
        
        self.particles = (0..self.parameters.swarm_size)
            .map(|_| Particle::new(dimension))
            .collect();
    }

    /// Evaluate fitness for all particles
    fn evaluate_all_particles(&mut self) {
        let courses = self.courses.clone();
        let checker = self.checker.clone();

        self.particles.par_iter_mut().for_each(|particle| {
            let schedule = Self::position_to_schedule(&particle.position, &courses);
            particle.fitness = checker.evaluate(&schedule);
            particle.update_personal_best();
        });
    }

    /// Update global best from all particles
    fn update_global_best(&mut self) {
        for particle in &self.particles {
            if particle.pbest_fitness < self.global_best_fitness && !particle.pbest_fitness.is_nan() {
                self.global_best_fitness = particle.pbest_fitness;
                self.global_best_position = particle.pbest_position.clone();
            }
        }
    }

    /// Update all particles (velocity and position)
    fn update_all_particles(&mut self) {
        // Clone global best for parallel access
        let global_best_position = self.global_best_position.clone();
        let params = self.parameters.clone();

        // Parallel particle updates
        self.particles.par_iter_mut().for_each(|particle| {
            particle.update_velocity(
                &global_best_position,
                params.inertia_weight,
                params.cognitive_weight,
                params.social_weight,
            );
            particle.update_position();
        });
    }

    /// Emit progress to frontend
    fn emit_progress(
        &self,
        window: Option<&Window>,
        iteration: usize,
        start_time: &Instant,
        all_best_fitness: &[f64],
        current_run: usize,
        total_runs: usize,
        is_finished: bool,
    ) {
        if let Some(window) = window {
            let _ = window.emit(
                "optimization-progress",
                OptimizationProgress {
                    iteration,
                    elapsed_time: start_time.elapsed(),
                    all_best_fitness: Some(all_best_fitness.to_vec()),
                    best_fitness: self.global_best_fitness,
                    current_run: Some(current_run),
                    total_runs: Some(total_runs),
                    is_finished,
                },
            );
        }
    }
    
    /// Convert particle position to valid schedule
    pub fn position_to_schedule(
        position: &[f64],
        courses: &[CourseRequest],
    ) -> Vec<OptimizedCourse> {
        let mut grouped: HashMap<(u32, u32, u32, u32), Vec<(f64, f64, OptimizedCourse)>> = HashMap::new();

        // Group courses by prodi, semester, class, and time
        for (i, course) in courses.iter().enumerate() {
            let idx = i * 2;
            if idx + 1 >= position.len() {
                break;
            }

            let day_order = position[idx];
            let time_order = position[idx + 1];
            let key = (course.prodi, course.semester, course.id_kelas, course.id_waktu);

            let opt_course = OptimizedCourse {
                id_jadwal: course.id_jadwal,
                id_matkul: course.id_matkul,
                id_dosen: course.id_dosen,
                id_kelas: course.id_kelas,
                id_waktu: course.id_waktu,
                hari: 0,
                jam_mulai: 0,
                jam_akhir: 0,
                ruangan: 0,
                semester: course.semester,
                sks: course.sks,
                prodi: course.prodi,
            };

            grouped.entry(key).or_default().push((day_order, time_order, opt_course));
        }

        let mut scheduled = Vec::with_capacity(courses.len());

        // Schedule days based on day_order
        for entries in grouped.into_values() {
            let mut sorted = entries;
            sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

            // SKS limit per day based on number of courses
            let max_sks = if sorted.len() == 4 { 3 } else { 6 };
            let mut sks_per_day = [0u32; 5]; // Monday-Friday
            let mut current_day = 0;

            for (_, time_order, mut course) in sorted {
                // Find available day
                while current_day < 5 {
                    if sks_per_day[current_day] + course.sks <= max_sks {
                        course.hari = current_day as u32 + 1; // 1=Monday, 2=Tuesday, etc.
                        sks_per_day[current_day] += course.sks;
                        break;
                    }
                    current_day += 1;
                }

                // Fallback to Friday if no slot available
                if course.hari == 0 {
                    course.hari = 5; // Friday
                }

                scheduled.push((
                    (course.prodi, course.semester, course.id_kelas, course.id_waktu, course.hari),
                    time_order,
                    course,
                ));
            }
        }

        // Schedule times based on time_order
        let mut by_day: HashMap<_, Vec<_>> = HashMap::new();
        for (key, time_order, course) in scheduled {
            by_day.entry(key).or_default().push((time_order, course));
        }

        let mut final_schedule = Vec::with_capacity(courses.len());

        for ((_, _, _, id_waktu, _), mut entries) in by_day {
            entries.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

            // Determine time range based on id_waktu
            let (start, end) = match id_waktu {
                1 => (480, 720),   // Morning: 08:00-12:00 (in minutes)
                2 => (1080, 1320), // Evening: 18:00-22:00 (in minutes)
                _ => (480, 720),   // Default morning
            };

            let mut current_time = start;

            for (_, mut course) in entries {
                let duration = course.sks * 40; // 40 minutes per SKS
                
                // Reset to start if not enough time
                if current_time + duration > end {
                    current_time = start;
                }

                course.jam_mulai = current_time;
                course.jam_akhir = current_time + duration;
                current_time += duration;

                final_schedule.push(course);
            }
        }

        final_schedule
    }
}