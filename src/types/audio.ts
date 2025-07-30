// types/audio.ts

export interface AudioDevice {
	name: string;
	index: number;
	is_default?: boolean;
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

// Events from Tauri backend
export interface SpectrumUpdateEvent {
	spectrum: number[];
	timestamp?: number;
}

export interface AudioStatusEvent {
	status: 'started' | 'stopped' | 'error' | 'starting';
	message: string;
}

export interface GainChangedEvent {
	gain: number;
	timestamp?: number;
}

// API Response types
export interface AudioDevicesResponse {
	devices: AudioDevice[];
	count: number;
	default_device?: string;
}

export interface AudioStatusResponse {
	running: boolean;
	gain: number;
	spectrum_size: number;
	has_signal: boolean;
	fft_size: number;
	spectrum_bins: number;
	sample_rate: number;
	channels: number;
}

export interface AudioSpectrumResponse {
	spectrum: number[];
	max: number;
	average: number;
	size: number;
}
