import { defineStore } from 'pinia';
import { computed, ref, watch } from 'vue';

import { COLOR_MODES, DEFAULT_CUSTOM_COLOR } from '@/config';
import type { ColorConfig, ColorMode, CustomColor } from '@/types';

export const useColorsStore = defineStore('colors', () => {
	// ===== STATE =====
	const currentMode = ref<string>('rainbow');
	const customColor = ref<CustomColor>({ ...DEFAULT_CUSTOM_COLOR });
	const availableModes = ref<ColorMode[]>([...COLOR_MODES]);
	const loading = ref(false);

	// Force reactivity with watchers
	watch(currentMode, (newMode) => {
		console.log(`🌈 [COLORS_STORE] Mode changed to: ${newMode}`);
	});

	watch(
		customColor,
		(newColor) => {
			console.log(`🎨 [COLORS_STORE] Custom color changed:`, newColor);
		},
		{ deep: true }
	);

	// ===== GETTERS =====
	const colorPreviewStyle = computed(() => {
		const { r, g, b } = customColor.value;
		const rgb = [r, g, b].map((v) => Math.round(Math.max(0, Math.min(255, v * 255))));
		return {
			backgroundColor: `rgb(${rgb.join(', ')})`,
		};
	});

	const hexColor = computed(() => {
		const { r, g, b } = customColor.value;
		const hex = [r, g, b]
			.map((v) => Math.round(Math.max(0, Math.min(255, v * 255))))
			.map((v) => v.toString(16).padStart(2, '0'))
			.join('');
		return `#${hex}`.toUpperCase();
	});

	const currentModeInfo = computed(() => availableModes.value.find((mode) => mode.value === currentMode.value));

	const isCustomMode = computed(() => currentMode.value === 'custom');

	// ===== ACTIONS =====
	const setCurrentMode = (mode: string) => {
		console.log(`🌈 [COLORS_STORE] Setting mode: ${currentMode.value} → ${mode}`);
		currentMode.value = mode;
	};

	const setCustomColor = (color: CustomColor) => {
		const validColor = {
			r: Math.max(0, Math.min(1, color.r || 0)),
			g: Math.max(0, Math.min(1, color.g || 0)),
			b: Math.max(0, Math.min(1, color.b || 0)),
		};

		console.log(`🎨 [COLORS_STORE] Setting color:`, {
			from: customColor.value,
			to: validColor,
		});

		customColor.value = validColor;
	};

	const setLoading = (isLoading: boolean) => {
		console.log(`⏳ [COLORS_STORE] Loading: ${loading.value} → ${isLoading}`);
		loading.value = isLoading;
	};

	const updateColorConfig = (config: ColorConfig) => {
		console.log(`🔄 [COLORS_STORE] Updating config:`, config);

		if (config.mode && config.mode !== currentMode.value) {
			setCurrentMode(config.mode);
		}

		if (config.custom_color) {
			setCustomColor(config.custom_color);
		}

		if (config.available_modes && Array.isArray(config.available_modes)) {
			const updatedModes = config.available_modes.map((mode) => {
				const existing = COLOR_MODES.find((m) => m.value === mode);
				return (
					existing || {
						value: mode,
						label: mode.charAt(0).toUpperCase() + mode.slice(1),
						emoji: '🎨',
					}
				);
			});
			availableModes.value = updatedModes;
		}
	};

	const validateMode = (mode: string): boolean => {
		return availableModes.value.some((m) => m.value === mode);
	};

	const validateColor = (color: CustomColor): CustomColor => {
		return {
			r: Math.max(0, Math.min(1, color.r || 0)),
			g: Math.max(0, Math.min(1, color.g || 0)),
			b: Math.max(0, Math.min(1, color.b || 0)),
		};
	};

	// Force trigger reactivity
	const forceUpdate = () => {
		const current = { ...customColor.value };
		customColor.value = { ...current };
	};

	// Pinia $reset method
	const $reset = () => {
		console.log('🔄 [COLORS_STORE] Resetting store');
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
		setLoading,
		updateColorConfig,
		validateMode,
		validateColor,
		forceUpdate,
		$reset,
	};
});
