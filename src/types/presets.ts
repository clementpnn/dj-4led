import type { CustomColor } from '@/types/colors';

export interface Preset {
	id: string;
	name: string;
	description: string;
	config: PresetConfig;
	createdAt: number;
	updatedAt?: number;
	author?: string;
	tags?: string[];
}

export interface PresetConfig {
	effect: {
		id: number;
		name: string;
		parameters?: Record<string, any>;
	};
	color: {
		mode: string;
		customColor?: CustomColor;
		palette?: string;
	};
	audio: {
		gain: number;
		device?: string;
	};
	led: {
		brightness: number;
		mode: 'simulator' | 'production';
		controllers?: string[];
	};
}

export interface PresetCategory {
	id: string;
	name: string;
	description: string;
	presets: Preset[];
}
