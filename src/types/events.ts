export interface AppEvent {
	type: string;
	payload: any;
	timestamp: number;
	source?: string;
}

export interface AudioEvent extends AppEvent {
	type: 'audio:capture:start' | 'audio:capture:stop' | 'audio:gain:change' | 'audio:device:change';
	payload: {
		device?: string;
		gain?: number;
		spectrum?: number[];
	};
}

export interface EffectEvent extends AppEvent {
	type: 'effect:change' | 'effect:transition:start' | 'effect:transition:end';
	payload: {
		effectId: number;
		effectName: string;
		progress?: number;
	};
}

export interface LEDEvent extends AppEvent {
	type: 'led:start' | 'led:stop' | 'led:brightness:change' | 'led:pattern:test';
	payload: {
		mode?: 'simulator' | 'production';
		brightness?: number;
		pattern?: string;
	};
}
