export interface Effect {
	id: number;
	name: string;
	display_name?: string;
	description?: string;
	category?: string;
	emoji?: string;
}

export interface EffectState {
	id: number;
	name: string;
	transitioning: boolean;
	transition_progress: number;
}

export interface EffectInfo {
	id: number;
	name: string;
	description: string;
	performance_impact: 'low' | 'medium' | 'high';
	supports_transitions: boolean;
	supports_custom_colors: boolean;
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
