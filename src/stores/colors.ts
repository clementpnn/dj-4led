import { defineStore } from 'pinia';
import { computed, readonly, ref } from 'vue';
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
		const { r, g, b } = customColor.value;
		const rgb = [r, g, b].map((v) => Math.round(v * 255));
		return {
			backgroundColor: `rgb(${rgb.join(', ')})`,
		};
	});

	const hexColor = computed(() => {
		const { r, g, b } = customColor.value;
		const hex = [r, g, b]
			.map((v) => Math.round(v * 255))
			.map((v) => v.toString(16).padStart(2, '0'))
			.join('');
		return `#${hex}`.toUpperCase();
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
		currentMode: readonly(currentMode),
		customColor: readonly(customColor),
		availableModes: readonly(availableModes),
		loading: readonly(loading),

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
