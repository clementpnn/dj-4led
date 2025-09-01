import type { ColorChannel, ColorMode, Effect, TestPattern } from '@/types';

export const EFFECTS: Effect[] = [
	{
		id: 0,
		name: 'SpectrumBars',
		display_name: 'Spectrum Bars',
		emoji: '📊',
		description: 'Classic spectrum analyzer bars',
		category: 'spectrum',
	},
	{
		id: 1,
		name: 'CircularWave',
		display_name: 'Circular Wave',
		emoji: '🌊',
		description: 'Circular ripple effect from center',
		category: 'ambient',
	},
	{
		id: 2,
		name: 'ParticleSystem',
		display_name: 'Particle System',
		emoji: '✨',
		description: 'Dynamic particle effects',
		category: 'particle',
	},
	{
		id: 3,
		name: 'Heartbeat',
		display_name: 'Heartbeat',
		emoji: '💗',
		description: 'Pulsing heartbeat effect',
		category: 'rhythm',
	},
	{
		id: 4,
		name: 'Starfall',
		display_name: 'Starfall',
		emoji: '⭐',
		description: 'Falling stars effect',
		category: 'ambient',
	},
	{
		id: 5,
		name: 'Rain',
		display_name: 'Rain',
		emoji: '🌧️',
		description: 'Rain drops effect',
		category: 'ambient',
	},
	{
		id: 6,
		name: 'Flames',
		display_name: 'Flames',
		emoji: '🔥',
		description: 'Fire simulation',
		category: 'ambient',
	},
	{
		id: 7,
		name: 'Applaudimetre',
		display_name: 'Applaudimètre',
		emoji: '👏',
		description: 'Applause meter with peak detection',
		category: 'rhythm',
	},
];

export const COLOR_MODES: ColorMode[] = [
	{
		value: 'rainbow',
		label: 'Rainbow',
		emoji: '🌈',
		description: 'Smooth rainbow color transitions',
	},
	{
		value: 'fire',
		label: 'Fire',
		emoji: '🔥',
		description: 'Warm fire colors from red to yellow',
	},
	{
		value: 'ocean',
		label: 'Ocean',
		emoji: '🌊',
		description: 'Cool ocean colors from blue to cyan',
	},
	{
		value: 'sunset',
		label: 'Sunset',
		emoji: '🌅',
		description: 'Sunset colors from orange to purple',
	},
	{
		value: 'matrix',
		label: 'Matrix',
		emoji: '🌿',
		description: 'Green matrix style colors',
	},
	{
		value: 'custom',
		label: 'Custom',
		emoji: '🎨',
		description: 'Your own custom color',
	},
];

export const COLOR_CHANNELS: ColorChannel[] = [
	{ key: 'r', name: 'Red', emoji: '🔴', color: '#ff5555' },
	{ key: 'g', name: 'Green', emoji: '🟢', color: '#55ff55' },
	{ key: 'b', name: 'Blue', emoji: '🔵', color: '#5555ff' },
];

export const TEST_PATTERNS: TestPattern[] = [
	{
		value: 'red',
		label: 'Red',
		emoji: '🔴',
		description: 'Solid red color',
	},
	{
		value: 'green',
		label: 'Green',
		emoji: '🟢',
		description: 'Solid green color',
	},
	{
		value: 'blue',
		label: 'Blue',
		emoji: '🔵',
		description: 'Solid blue color',
	},
	{
		value: 'white',
		label: 'White',
		emoji: '⚪',
		description: 'Solid white color',
	},
	{
		value: 'gradient',
		label: 'Gradient',
		emoji: '🌈',
		description: 'Rainbow gradient pattern',
	},
	{
		value: 'checkerboard',
		label: 'Checkerboard',
		emoji: '🏁',
		description: 'Black and white checkerboard',
	},
	{
		value: 'quarter',
		label: 'Quarter Test',
		emoji: '🔳',
		description: 'Four colored quarters',
	},
];
