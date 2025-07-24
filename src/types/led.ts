export interface LEDStats {
	is_running: boolean;
	brightness: number;
	frame_size: number;
	matrix_size: string;
	controllers: number;
	mode: 'simulator' | 'production';
}

export interface LEDController {
	id: string;
	address: string;
	port: number;
	status: 'connected' | 'disconnected' | 'error';
	lastSeen?: number;
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
