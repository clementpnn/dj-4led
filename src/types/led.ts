export interface LEDStats {
	is_running: boolean;
	mode: 'simulator' | 'production';
	brightness: number;
	frame_size: number;
	matrix_size: string;
	target_fps?: number;
	frame_time_ms?: number;
	controllers: number;
}

export interface LEDController {
	id: string;
	name: string;
	ip: string;
	status: 'connected' | 'disconnected' | 'error';
	type: 'hardware' | 'simulator';
	lastSeen: number;
}

export interface LEDConfig {
	defaultBrightness: number;
	maxBrightness: number;
	refreshRate: number;
	matrixWidth: number;
	matrixHeight: number;
	colorOrder: 'RGB' | 'GRB' | 'BGR';
}

export interface TestPattern {
	value: string;
	label: string;
	emoji: string;
	description?: string;
}
