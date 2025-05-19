use std::collections::HashMap;
use std::time::Instant;

use tauri::{Emitter, Window};

use super::particle::Particle;
use super::fitness::FitnessCalculator;
use super::models::{
    CourseRequest, 
    TimePreferenceRequest, 
    PsoParameters, 
    OptimizedCourse, 
    ConflictInfo,
    OptimizationProgress,
    Status
};  



pub struct PSO {
    particles: Vec<Particle>,
    global_best_position: Vec<f32>,
    global_best_fitness: f32,
    parameters: PsoParameters,
    courses: Vec<CourseRequest>,
    fitness_calculator: FitnessCalculator,
}

impl PSO {
    pub fn new(
        courses: Vec<CourseRequest>,
        time_preferences: Vec<TimePreferenceRequest>,
        parameters: PsoParameters,
    ) -> Self {
        // Each course requires 3 values (day, time slot, room)
        let dimension = courses.len() * 2;
        
        // Create particles based on swarm size from parameters
        let particles = (0..parameters.swarm_size)
            .map(|_| Particle::new(dimension))
            .collect::<Vec<_>>();
        
        // Create fitness calculator
        let fitness_calculator = FitnessCalculator::new(time_preferences);
        
        PSO {
            particles,
            global_best_position: vec![0.0; dimension],
            global_best_fitness: f32::INFINITY,
            courses,
            fitness_calculator,
            parameters,
        }
    }
    
    pub async fn optimize(&mut self, window: &Window, run_info: Option<(usize, usize)>) -> Vec<f32> {
        let start_time = Instant::now();
        // let _ = window.emit("status-optimize", Status {
        //     message: "mulai".to_string()
        // });
        self.initialize_swarm();
        
        let courses = &self.courses;
        let params = &self.parameters;
        let (current_run, total_runs) = run_info.unwrap_or((0, 0));
    
        for iteration in 0..params.max_iterations {
    
            for particle in &mut self.particles {
                particle.update_velocity(
                    &self.global_best_position,
                    params.inertia_weight,
                    params.cognitive_weight,
                    params.social_weight,
                    // params.velocity_clamp,
                );
                particle.update_position();
    
                let schedule: Vec<_> = Self::position_to_schedule(&particle.position, courses);
                let (fitness, _) = self.fitness_calculator.calculate_fitness(&schedule);
    
                if fitness < particle.pbest_fitness {
                    particle.pbest_fitness = fitness;
                    particle.pbest_position = particle.position.clone();
                }
            }
    
            if let Some(best_particle) = self.particles.iter().min_by(|a, b| {
                a.pbest_fitness.partial_cmp(&b.pbest_fitness).unwrap()
            }) {
                if best_particle.pbest_fitness < self.global_best_fitness {
                    self.global_best_fitness = best_particle.pbest_fitness;
                    self.global_best_position = best_particle.pbest_position.clone();
                }
            }
    
            // let schedule = Self::position_to_schedule(&self.global_best_position, courses);
            // let (_, current_conflicts) = self.fitness_calculator.calculate_fitness(&schedule);

            let _ = window.emit("optimization-progress", OptimizationProgress {
                iteration: iteration + 1,
                elapsed_time: start_time.elapsed(),
                all_best_fitness: None,
                best_fitness: self.global_best_fitness,
                current_run: if run_info.is_some() { Some(current_run) } else { None },
                total_runs: if run_info.is_some() { Some(total_runs) } else { None },
                is_finished: current_run + 1 >= total_runs,
                conflicts: ConflictInfo::default(),
            });
        
        }

        let _ = window.emit("optimization-progress", OptimizationProgress {
            iteration: params.max_iterations,
            elapsed_time: start_time.elapsed(),
            all_best_fitness: None,
            best_fitness: self.global_best_fitness,
            current_run: if run_info.is_some() { Some(current_run) } else { None },
            total_runs: if run_info.is_some() { Some(total_runs) } else { None },
            is_finished: true,
            conflicts: ConflictInfo::default(),
        });
    
        self.global_best_position.clone()
    }
    
    // Initialize all particles and find initial global best
    fn initialize_swarm(&mut self) {
        let courses = &self.courses;
        let fitness_calculator = &self.fitness_calculator;
        
        for particle in &mut self.particles {
            let schedule = Self::position_to_schedule(&particle.position, courses);
            let (fitness, _) = fitness_calculator.calculate_fitness(&schedule);
            
            // Update personal best
            if fitness < particle.pbest_fitness {
                particle.pbest_fitness = fitness;
                particle.pbest_position = particle.position.clone();
            }
            
            // Update global best
            if fitness < self.global_best_fitness {
                self.global_best_fitness = fitness;
                self.global_best_position = particle.position.clone();
            }
        }
    }

  

    pub fn position_to_schedule(
        position: &[f32],
        courses: &[CourseRequest],
    ) -> Vec<OptimizedCourse> {
        let mut schedule_entries = Vec::with_capacity(courses.len());
        let mut room_allocation = HashMap::new();
        let mut current_room = 1;
    
        // Alokasikan ruangan
        for course in courses {
            room_allocation
                .entry((course.prodi, course.semester, course.id_kelas))
                .or_insert_with(|| {
                    let room = current_room;
                    current_room += 1;
                    room
                });
        }
    
        // Buat entry awal jadwal
        for (i, course) in courses.iter().enumerate() {
            let base_idx = i * 2;
            if base_idx + 1 >= position.len() {
                break;
            }
    
            let day_order = position[base_idx];
            let urutan = position[base_idx + 1];
    
            let ruangan = *room_allocation
                .get(&(course.prodi, course.semester, course.id_kelas))
                .unwrap_or(&1);
    
            schedule_entries.push((
                (course.prodi, course.semester, course.id_kelas, course.id_waktu),
                day_order,
                urutan,
                OptimizedCourse {
                    id_jadwal: course.id_jadwal,
                    id_matkul: course.id_matkul,
                    id_dosen: course.id_dosen,
                    id_kelas: course.id_kelas,
                    id_waktu: course.id_waktu,
                    hari: 0,
                    jam_mulai: 0,
                    jam_akhir: 0,
                    ruangan,
                    semester: course.semester,
                    sks: course.sks,
                    prodi: course.prodi,
                },
            ));
        }
    
        // Kelompokkan berdasarkan group (prodi, semester, kelas, waktu)
        let mut grouped: HashMap<(u32, u32, u32, u32), Vec<(f32, f32, OptimizedCourse)>> = HashMap::new();

        for (key, day_order, urutan, course) in schedule_entries {
            grouped.entry(key).or_default().push((day_order, urutan, course));
        }
    
        let mut scheduled_with_day = Vec::with_capacity(courses.len());
    
        for mut entries in grouped.into_values() {
            entries.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            let mut sks_per_day = [0u32; 5];
            let mut current_day = 0;
    
            for (_, urutan, mut course) in entries {
                while current_day < 5 {
                    if sks_per_day[current_day] + course.sks <= 6 {
                        course.hari = current_day as u32 + 1;
                        sks_per_day[current_day] += course.sks;
                        break;
                    }
                    current_day += 1;
                }
    
                if course.hari == 0 {
                    course.hari = 5;
                }
    
                scheduled_with_day.push((
                    (course.prodi, course.semester, course.id_kelas, course.id_waktu, course.hari),
                    urutan,
                    course,
                ));
            }
        }
    
        // Kelompokkan berdasarkan hari
        let mut by_day: HashMap<(u32, u32, u32, u32, u32), Vec<(f32, OptimizedCourse)>> = HashMap::new();        for (key, urutan, course) in scheduled_with_day {
            by_day.entry(key).or_default().push((urutan, course));
        }
    
        let mut final_schedule = Vec::with_capacity(courses.len());
    
        for ((_, _, _, id_waktu, _), mut entries) in by_day {
            entries.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    
            let (start, end) = match id_waktu {
                1 => (480, 720),
                2 => (1080, 1320),
                _ => (480, 720),
            };
    
            let mut current_time = start;
    
            for (_, mut course) in entries {
                let durasi = course.sks * 40;
                if current_time + durasi > end {
                    current_time = start;
                }
    
                course.jam_mulai = current_time;
                course.jam_akhir = current_time + durasi;
                current_time += durasi;
    
                final_schedule.push(course);
            }
        }
    
        final_schedule
    }
    
    
    // Method to evaluate best position
    pub fn evaluate_best_position(&self) -> (f32, ConflictInfo) {
        let schedule = Self::position_to_schedule(&self.global_best_position, &self.courses);
        self.fitness_calculator.calculate_fitness(&schedule)
    }
}