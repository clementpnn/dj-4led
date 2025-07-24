export interface Effect {
	id: number;
	name: string;
	display_name: string;
	emoji: string;
	description: string;
	category?: 'spectrum' | 'particle' | 'rhythm' | 'ambient';
	supports_transitions?: boolean;
	performance_impact?: 'low' | 'medium' | 'high';
}

export interface EffectState {
	id: number;
	name: string;
	transitioning: boolean;
	transition_progress: number;
}

export interface EffectParameter {
	key: string;
	name: string;
	type: 'number' | 'boolean' | 'color' | 'select';
	value: number | boolean | string;
	min?: number;
	max?: number;
	step?: number;
	options?: string[];
}

export interface EffectInfo {
	id: number;
	name: string;
	description: string;
	supports_transitions: boolean;
	supports_custom_colors: boolean;
	performance_impact: string;
	parameters?: EffectParameter[];
}
