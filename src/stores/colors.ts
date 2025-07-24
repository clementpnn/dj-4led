import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { COLOR_MODES, DEFAULT_CUSTOM_COLOR } from '../config';
import type { ColorConfig, ColorMode, CustomColor } from '../types';

export const useColorsStore = defineStore('colors', () => {
	// ===== STATE =====

	const currentMode = ref<string>('rainbow');
	const customColor = ref<CustomColor>({ ...DEFAULT_CUSTOM_COLOR });
	const availableModes = ref<ColorMode[]>([...COLOR_MODES]);
	const loading = ref(false);

	// ===== GETTERS =====

	const colorPreviewStyle = computed(() => {
		const r = Math.round(customColor.value.r * 255);
		const g = Math.round(customColor.value.g * 255);
		const b = Math.round(customColor.value.b * 255);
		return {
			backgroundColor: `rgb(${r}, ${g}, ${b})`,
		};
	});

	const hexColor = computed(() => {
		const r = Math.round(customColor.value.r * 255);
		const g = Math.round(customColor.value.g * 255);
		const b = Math.round(customColor.value.b * 255);
		return `#${r.toString(16).padStart(2, '0')}${g.toString(16).padStart(2, '0')}${b.toString(16).padStart(2, '0')}`.toUpperCase();
	});

	const currentModeInfo = computed(() => availableModes.value.find((mode) => mode.value === currentMode.value));

	const isCustomMode = computed(() => currentMode.value === 'custom');

	// ===== ACTIONS =====

	const setCurrentMode = (mode: string) => {
		if (availableModes.value.some((m) => m.value === mode)) {
			currentMode.value = mode;
		}
	};

	const setCustomColor = (color: CustomColor) => {
		// Clamp values between 0 and 1
		customColor.value = {
			r: Math.max(0, Math.min(1, color.r)),
			g: Math.max(0, Math.min(1, color.g)),
			b: Math.max(0, Math.min(1, color.b)),
		};
	};

	const setAvailableModes = (modes: ColorMode[]) => {
		availableModes.value = modes;
	};

	const setLoading = (isLoading: boolean) => {
		loading.value = isLoading;
	};

	const updateColorConfig = (config: ColorConfig) => {
		currentMode.value = config.mode;
		if (config.custom_color) {
			customColor.value = { ...config.custom_color };
		}
		if (config.available_modes) {
			// Merge with existing modes, keeping emojis and descriptions
			const updatedModes = config.available_modes.map((mode) => {
				const existing = availableModes.value.find((m) => m.value === mode);
				return existing || { value: mode, label: mode, emoji: '🎨' };
			});
			availableModes.value = updatedModes;
		}
	};

	const reset = () => {
		currentMode.value = 'rainbow';
		customColor.value = { ...DEFAULT_CUSTOM_COLOR };
		availableModes.value = [...COLOR_MODES];
		loading.value = false;
	};

	return {
		// State
		currentMode,
		customColor,
		availableModes,
		loading,

		// Getters
		colorPreviewStyle,
		hexColor,
		currentModeInfo,
		isCustomMode,

		// Actions
		setCurrentMode,
		setCustomColor,
		setAvailableModes,
		setLoading,
		updateColorConfig,
		reset,
	};
});
