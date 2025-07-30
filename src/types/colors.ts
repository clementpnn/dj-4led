export interface CustomColor {
	r: number;
	g: number;
	b: number;
}

export interface ColorMode {
	value: string;
	label: string;
	emoji?: string;
	description?: string;
}

export interface ColorConfig {
	mode: string;
	custom_color?: CustomColor;
	available_modes?: string[];
}
export interface ColorChannel {
	key: 'r' | 'g' | 'b';
	name: string;
	emoji: string;
	color: string;
}

export interface ColorPalette {
	name: string;
	colors: CustomColor[];
	description?: string;
}
