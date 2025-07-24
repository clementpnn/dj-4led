export interface SystemStats {
	audio: {
		is_capturing: boolean;
		gain: number;
		spectrum_size: number;
		device_count: number;
	};
	effects: {
		current_effect: string;
		transitioning: boolean;
		available_effects: number;
	};
	led: {
		is_running: boolean;
		brightness: number;
		controllers: number;
		frame_rate: number;
	};
	performance: {
		fps: number;
		frame_count: number;
		uptime: number;
		memory_usage?: number;
		cpu_usage?: number;
	};
}

export interface SystemHealth {
	status: 'healthy' | 'warning' | 'critical' | 'unknown';
	issues: string[];
	score: number;
	lastCheck: number;
}

export interface SystemConfig {
	monitoringInterval: number;
	healthCheckInterval: number;
	autoRestart: boolean;
	maxMemoryUsage: number;
	logLevel: 'debug' | 'info' | 'warn' | 'error';
}
