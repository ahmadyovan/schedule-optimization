import { useEffect, useState } from "react";
import { listen } from '@tauri-apps/api/event';
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

type ConflictInfo = {
  conflicts_list: any[]; // Ganti `any` dengan tipe yang sesuai jika kamu tahu struktur konflik
  group_conflicts: any[];
  preference_conflicts: any[];
  total_conflicts: number;
};


type ElapsedTime = {
  secs: number;
  nanos: number;
};

export type OptimizationProgress = {
  all_best_fitness: number | null;
  best_fitness: number;
  conflicts: ConflictInfo;
  current_run: number | null;
  elapsed_time: ElapsedTime;
  is_finished: boolean;
  iteration: number;
  total_runs: number | null;
};

const App = () => {
	const [courseData, setCourseData] = useState("");
	const [preferenceData, setPreferenceData] = useState("");
	const [courseFileName, setCourseFileName] = useState<string | null>(null);
  	const [preferenceFileName, setPreferenceFileName] = useState<string | null>(null);
	const [isOpen, setIsOpen] = useState(false);
	const [isRunning, setIsRunning] = useState(false);
	const [params, setParams] = useState({
		swarm_size: 30,
		max_iterations: 100,
		cognitive_weight: 2.0,
		social_weight: 2.0,
		inertia_weight: 0.7,
		num_runs: 1,
	});

	const [progress, setProgress] = useState<OptimizationProgress>({
		all_best_fitness: null,
		best_fitness: 0,
		conflicts: {
			conflicts_list: [],
			group_conflicts: [],
			preference_conflicts: [],
			total_conflicts: 0,
		},
		current_run: null,
		elapsed_time: {
			secs: 0,
			nanos: 0,
		},
		is_finished: false,
		iteration: 0,
		total_runs: null,
	});


	useEffect(() => {
 		const unlistenPromise = listen<OptimizationProgress>('optimization-progress', (event) => {
		const data = event.payload;

		console.log('Progress:', data);

		setProgress({
			iteration: data.iteration,
			best_fitness: data.best_fitness,
			conflicts: {
				conflicts_list: data.conflicts.conflicts_list || [],
				group_conflicts: data.conflicts.group_conflicts || [],
				preference_conflicts: data.conflicts.preference_conflicts || [],
				total_conflicts: data.conflicts.total_conflicts ?? 0,
			},
			all_best_fitness: data.all_best_fitness ?? null,
			current_run: data.current_run ?? null,
			elapsed_time: {
				secs: data.elapsed_time?.secs ?? 0,
				nanos: data.elapsed_time?.nanos ?? 0,
			},
			is_finished: data.is_finished ?? false,
			total_runs: data.total_runs ?? null,
			});
		});

		return () => {
			// Pastikan unlisten dipanggil ketika komponen dibersihkan
			unlistenPromise.then((unlisten) => unlisten());
		};
	}, []);

	const handleFileChange = async (e: any, setData: any, setFileName: any) => {
		const file = e.target.files[0];
		if (!file) return;
		const text = await file.text();
		setFileName(file.name)
		setData(text);			
	};


	const showNotification = (message: string, type: any) => {
		const container = document.getElementById("notification-container");
		if (!container) return;

		const notification = document.createElement("div");
		notification.className = `bg-red-500 text-white px-4 py-2 rounded shadow-md mb-2`;
		notification.textContent = message;
		container.appendChild(notification);

		setTimeout(() => {
			notification.remove();
		}, 10000);
	};

	const runOptimization = async () => {
		if (!courseData) return showNotification("Please upload course data first", "error");
		if (!preferenceData) return showNotification("Please upload time preferences", "warning");

		try {
			showNotification("Starting optimization...", "info");
			setIsRunning(true)
			const result = await invoke("process_pso", {
				courseCsv: courseData,
				preferenceCsv: preferenceData,
				params: params,
			});
			console.log(result);
			showNotification("Optimization completed!", "success");
			// setIsRunning(false)
		} catch (err) {
			showNotification(`Error: ${err}`, "error");
			setIsRunning(false)
		}
	};

  return (
    <div className="h-screen w-screen flex flex-col">
		<div className="h-full flex flex-col gap-5 pt-10 items-center">
			<h1 className="w-fit text-2xl">OPTIMASI JADWAL KULIAH</h1>
			<div className="relative bg-green-300 w-1/2 rounded-lg px-8 py-10 flex flex-col justify-center items-center gap-5">
				<div className="absolute top-0 w-full text-center h-7" id="notification-container" />
				<div className="flex flex-col gap-2 items-center w-full">
					<p>masukan file jadwal kuliah</p>
					<div className={`flex gap-5 w-full ${courseFileName? "justify-start" : "justify-center"}`}>
						<label className="min-w-fit cursor-pointer" htmlFor="courseCSV">Upload CSV</label>
						<input className="hidden" id="courseCSV" type="file" accept=".csv" onChange={(e) => handleFileChange(e, setCourseData, setCourseFileName)} />
						{courseFileName && (
						<p className="text-sm flex items-center truncate text-nowrap text-gray-600">📄 {courseFileName}</p>
						)}
					</div>
				</div>
				<div className="flex flex-col gap-2 items-center w-full">
					<p>masukan file preferensi dosen</p>
					<div className={`flex gap-5 w-full ${preferenceFileName? "justify-start" : "justify-center"}`}>
						<label className="min-w-fit cursor-pointer" htmlFor="preferenceCSV">Upload CSV</label>
						<input className="hidden" id="preferenceCSV" type="file" accept=".csv" onChange={(e) => handleFileChange(e, setPreferenceData, setPreferenceFileName)} />
						{preferenceFileName && (
						<p className="text-sm flex items-center truncate text-gray-600">📄 {preferenceFileName}</p>
						)}
					</div>
				</div>
				<div className="flex flex-col gap-2 items-center">
					<p>jumlah percobaan</p>
					<input className="bg-white shadow-[0_2px_2px_rgba(0,0,0,0.2)] px-4 py-2 rounded-md" type="number" value={params.num_runs} onChange={(e) => setParams({ ...params, num_runs: +e.target.value})} />
				</div>
				<div className="w-full flex pt-8 justify-center">
					<button onClick={() => setIsOpen(true)}>optimasi</button>
				</div>
			</div>
		</div>

		{isOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-white/30 backdrop-blur-3xl">
          <div className="bg-white rounded-xl shadow-lg max-w-md w-full p-6">
            <h2 className="text-lg font-semibold mb-4">Parameter</h2>
            <div className="flex flex-col gap-3">
				<p>Jumlah Partikel</p>
				<input className="bg-gray-100 px-4 py-2 rounded-md" type="number" value={params.swarm_size} onChange={(e) => setParams({ ...params, swarm_size: +e.target.value })} />
				<p>Jumlah Iterasi</p>
				<input className="bg-gray-100 px-4 py-2 rounded-md" type="number" value={params.max_iterations} onChange={(e) => setParams({ ...params, max_iterations: +e.target.value })} />
				<p>Inertia Weight</p>
				<input className="bg-gray-100 px-4 py-2 rounded-md" type="number" value={params.inertia_weight} onChange={(e) => setParams({ ...params, inertia_weight: +e.target.value })} />
				<p>Cognitive Weight</p>
				<input className="bg-gray-100 px-4 py-2 rounded-md" type="number" value={params.cognitive_weight} onChange={(e) => setParams({ ...params, cognitive_weight: +e.target.value })} />
				<p>Social Weight</p>
				<input className="bg-gray-100 px-4 py-2 rounded-md" type="number" value={params.social_weight} onChange={(e) => setParams({ ...params, social_weight: +e.target.value })} />
			</div>
            <div className="flex justify-end">
				<button onClick={() => {runOptimization(); setIsOpen(false)}}>Run Optimization</button>
				<button onClick={() => setIsOpen(false)}>Tutup</button>
            </div>
          </div>
        </div>
      )}

	  {isRunning && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-white/30 backdrop-blur-3xl">
          <div className="bg-white rounded-xl shadow-lg min-w-xl max-w-1/2 w-full p-6">
            <h2 className="text-lg text-center font-semibold mb-4">Proses</h2>
			<div className="flex flex-col gap-3">
				<div className="grid grid-cols-[2fr_1fr_2fr]">
					<div>
						<h3>jumlah partikel</h3>
						<h3>jumlah iterasi</h3>
						<h3>inertia weight</h3>
						<h3>cognitive weight</h3>
						<h3>social weight</h3>
					</div>
					<div>
						<p>:</p>
						<p>:</p>
						<p>:</p>
						<p>:</p>
						<p>:</p>
					</div>
					<div>
						<p>{params.swarm_size}</p>
						<p>{params.max_iterations}</p>
						<p>{params.inertia_weight}</p>
						<p>{params.cognitive_weight}</p>
						<p>{params.social_weight}</p>
					</div>
				</div>
				<div className="grid grid-cols-[2fr_1fr_2fr]">
					<div>
						<h3>iterasi saat ini</h3>
						<h3>global best fitness</h3>
						<h3>jumlah konflik</h3>
						<h3>waktu</h3>
						<h3>jumlah pengujian</h3>
					</div>
					<div>
						<p>:</p>
						<p>:</p>
						<p>:</p>
						<p>:</p>
						<p>:</p>
					</div>
					<div>
						<p>{progress.iteration}</p>
						<p>{progress.best_fitness}</p>
						<p>{progress.conflicts.total_conflicts}</p>
						<p>
						{progress.elapsed_time
							? (() => {
								const totalSeconds =
								progress.elapsed_time.secs + progress.elapsed_time.nanos / 1_000_000_000;
								const minutes = Math.floor(totalSeconds / 60);
								const seconds = (totalSeconds % 60).toFixed(2);
								return `${minutes} menit ${seconds} detik`;
							})()
							: 'Belum tersedia'}
						</p>
						<p>{progress.current_run? progress.current_run + 1 : 0}</p>
					</div>
				</div>
			</div>
            <div className="flex justify-end pt-10">
				<button onClick={() => {runOptimization(); setIsOpen(false)}}>lagi</button>
				<button onClick={() => setIsRunning(false)}>Tutup</button>
            </div>
          </div>
        </div>
      )}
      
    </div>
  );
}

export default App;