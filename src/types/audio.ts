export interface AudioDevice {
	name: string;
	index: number;
}

export interface AudioState {
	isCapturing: boolean;
	devices: string[];
	currentGain: number;
	spectrum: number[];
	error: string | null;
}

export interface AudioStats {
	sampleRate: number;
	channels: number;
	bufferSize: number;
	inputLatency: number;
	outputLatency: number;
}

export interface AudioConfig {
	defaultGain: number;
	minGain: number;
	maxGain: number;
	spectrumBands: number;
	smoothingFactor: number;
}
