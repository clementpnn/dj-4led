import type { AudioConfig, CustomColor, LEDConfig, SystemConfig } from '../types';

export const DEFAULT_AUDIO_CONFIG: AudioConfig = {
	defaultGain: 1.0,
	minGain: 0.1,
	maxGain: 5.0,
	spectrumBands: 64,
	smoothingFactor: 0.7,
};

export const DEFAULT_LED_CONFIG: LEDConfig = {
	defaultBrightness: 0.8,
	maxBrightness: 1.0,
	refreshRate: 60,
	matrixWidth: 128,
	matrixHeight: 128,
	colorOrder: 'RGB',
};

export const DEFAULT_SYSTEM_CONFIG: SystemConfig = {
	monitoringInterval: 3000,
	healthCheckInterval: 5000,
	autoRestart: false,
	maxMemoryUsage: 512, // MB
	logLevel: 'info',
};

export const DEFAULT_CUSTOM_COLOR: CustomColor = {
	r: 1.0,
	g: 0.5,
	b: 0.0,
};

export const DEFAULT_PRESET_COLORS: CustomColor[] = [
	{ r: 1, g: 0, b: 0 }, // Rouge
	{ r: 0, g: 1, b: 0 }, // Vert
	{ r: 0, g: 0, b: 1 }, // Bleu
	{ r: 1, g: 1, b: 0 }, // Jaune
	{ r: 1, g: 0, b: 1 }, // Magenta
	{ r: 0, g: 1, b: 1 }, // Cyan
	{ r: 1, g: 0.5, b: 0 }, // Orange
	{ r: 1, g: 0.5, b: 0.8 }, // Rose
];
