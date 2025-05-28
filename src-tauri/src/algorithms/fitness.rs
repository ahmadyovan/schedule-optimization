use hashbrown::{HashMap};
use super::models::{
    TimePreferenceRequest, 
    OptimizedCourse, 
};


#[derive(Debug, Clone)]
pub struct FitnessCalculator {
    time_preferences: HashMap<u32, TimePreferenceRequest>,
}

impl FitnessCalculator {
    pub fn new(time_preferences: Vec<TimePreferenceRequest>) -> Self {
        Self {
            time_preferences: time_preferences.into_iter()
                .map(|p| (p.id_dosen, p))
                .collect(),
        }
    }

    pub fn calculate_fitness(&self, schedule: &[OptimizedCourse]) -> f32 {
        
        let distribution_penalty = self.check_schedule_distribution(schedule);
        let conflict_penalty = self.detect_conflicts(schedule);
        let pref_penalty = self.check_preferences(schedule);
        
        let total_penalty = conflict_penalty + distribution_penalty + pref_penalty;

                    
        (total_penalty as f32)
    }

    pub fn detect_conflicts(&self,schedule: &[OptimizedCourse]) -> u32 {
        let mut total_penalty = 0;
        let mut processed_pairs = std::collections::HashSet::new();
    
        for i in 0..schedule.len() {
            for j in (i + 1)..schedule.len() {
                let a = &schedule[i];
                let b = &schedule[j];
    
                // Lewati jika hari berbeda
                if a.hari != b.hari {
                    continue;
                }
    
                // Lewati jika tidak overlap
                if !Self::is_overlap(a, b) {
                    continue;
                }
    
                // Cek apakah bentrok karena dosen atau ruangan + kelas
                let same_ruangan_kelas = a.prodi == b.prodi && a.semester == b.semester && a.id_kelas == b.id_kelas;
                let same_dosen = a.id_dosen == b.id_dosen;
    
                if same_ruangan_kelas || same_dosen {
                    // Hindari duplikasi konflik
                    let key = (a.id_jadwal.min(b.id_jadwal), a.id_jadwal.max(b.id_jadwal));
                    if processed_pairs.contains(&key) {
                        continue;
                    }
                    processed_pairs.insert(key);
    
                    total_penalty += 100;
                }
            }
        }
    
        (total_penalty)
    }  


    fn check_schedule_distribution(&self, schedule: &[OptimizedCourse]) -> u32 {
        let mut counts = HashMap::new();   // (prodi, semester, id_waktu, hari) -> jumlah pelajaran
        let mut sks_counts = HashMap::new(); // (id_waktu, ruangan, hari) -> total SKS
    
        for course in schedule {
            let distrib_key = (course.prodi, course.semester, course.id_waktu, course.hari);
            *counts.entry(distrib_key).or_insert(0) += 1;
    
            let sks_key = (course.id_waktu, course.prodi, course.semester, course.id_kelas, course.hari);
            *sks_counts.entry(sks_key).or_insert(0) += course.sks;
        }
        let mut penalty = 0;   
        // Cek distribusi tidak merata
        let mut grouped = HashMap::new(); // (prodi, semester, id_waktu) -> Vec<count_per_day>
    
        for ((prodi, semester, waktu, _hari), count) in &counts {
            grouped.entry((prodi, semester, waktu))
                .or_insert(Vec::new())
                .push(*count);
        }

        // Cek limit SKS
        for ((_, _, _, _, _), total_sks) in sks_counts {
            if total_sks > 6 {
                let excess = total_sks - 6;
                let current_penalty = 100 * excess;
                penalty += current_penalty;
            }
        }
    
        penalty
    }


    // Optimasi 5: Preference check dengan lookup table
    fn check_preferences(&self, schedule: &[OptimizedCourse]) -> u32 {
        let hari_map = ["Senin", "Selasa", "Rabu", "Kamis", "Jumat"];
        let mut penalty = 0;
        let mut messages = Vec::new();

        for course in schedule {
            if let Some(pref) = self.time_preferences.get(&course.id_dosen) {
                let hari_idx = (course.hari - 1) as usize;
                let waktu_ok = if course.jam_mulai < 1080 {
                    match hari_idx {
                        0 => pref.seninPagi,
                        1 => pref.selasaPagi,
                        2 => pref.rabuPagi,
                        3 => pref.kamisPagi,
                        4 => pref.jumatPagi,
                        _ => false,
                    }
                } else {
                    match hari_idx {
                        0 => pref.seninMalam,
                        1 => pref.selasaMalam,
                        2 => pref.rabuMalam,
                        3 => pref.kamisMalam,
                        4 => pref.jumatMalam,
                        _ => false,
                    }
                };

                if !waktu_ok {
                    penalty += 300;
                    messages.push(format!(
                        "Preferensi: Dosen {} tidak suka {} {} untuk jadwal {}",
                        course.id_dosen,
                        hari_map[hari_idx],
                        if course.jam_mulai < 1080 { "Pagi" } else { "Malam" },
                        course.id_jadwal
                    ));
                }
            }
        }

        penalty
    }

    #[inline]
    fn is_overlap(a: &OptimizedCourse, b: &OptimizedCourse) -> bool {
        a.jam_mulai < b.jam_akhir && b.jam_mulai < a.jam_akhir
    }

}