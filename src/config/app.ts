export const APP_CONFIG = {
	name: 'DJ-4LED',
	version: '2.0.0',
	description: 'Professional LED Audio Visualizer',

	// UI Configuration
	ui: {
		theme: 'dark',
		animations: true,
		autoSave: true,
		compactMode: false,
	},

	// Performance
	performance: {
		maxLogs: 100,
		frameHistorySize: 10,
		spectrumUpdateRate: 60,
		uiUpdateRate: 30,
	},

	// Features
	features: {
		presets: true,
		advancedEffects: true,
		multiController: true,
		remoteControl: false,
	},

	// Debug
	debug: {
		enableConsole: true,
		verboseLogging: false,
		showPerformanceMetrics: false,
	},
} as const;
