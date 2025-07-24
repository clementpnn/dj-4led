import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { DEFAULT_LED_CONFIG } from '../config';
import type { LEDController, LEDStats } from '../types';

export const useLEDStore = defineStore('led', () => {
	// ===== STATE =====

	const stats = ref<LEDStats | null>(null);
	const controllers = ref<LEDController[]>([]);
	const brightness = ref<number>(DEFAULT_LED_CONFIG.defaultBrightness);
	const loading = ref(false);

	// ===== GETTERS =====

	const isRunning = computed(() => stats.value?.is_running || false);

	const currentMode = computed(() => stats.value?.mode || 'simulator');

	const frameSize = computed(() => stats.value?.frame_size || 0);

	const matrixSize = computed(() => stats.value?.matrix_size || '128x128');

	const controllerCount = computed(() => stats.value?.controllers || 0);

	const connectedControllers = computed(() => controllers.value.filter((c) => c.status === 'connected'));

	const isHealthy = computed(() => isRunning.value && brightness.value > 0 && connectedControllers.value.length > 0);

	// ===== ACTIONS =====

	const setStats = (newStats: LEDStats | null) => {
		stats.value = newStats;
		if (newStats) {
			brightness.value = newStats.brightness;
		}
	};

	const setControllers = (newControllers: LEDController[]) => {
		controllers.value = newControllers;
	};

	const updateController = (controllerId: string, updates: Partial<LEDController>) => {
		const index = controllers.value.findIndex((c) => c.id === controllerId);
		if (index !== -1) {
			controllers.value[index] = { ...controllers.value[index], ...updates };
		}
	};

	const setBrightness = (newBrightness: number) => {
		const clampedBrightness = Math.max(0, Math.min(1, newBrightness));
		brightness.value = clampedBrightness;
		if (stats.value) {
			stats.value.brightness = clampedBrightness;
		}
	};

	const setRunning = (running: boolean) => {
		if (stats.value) {
			stats.value.is_running = running;
		}
	};

	const setMode = (mode: 'simulator' | 'production') => {
		if (stats.value) {
			stats.value.mode = mode;
		}
	};

	const setLoading = (isLoading: boolean) => {
		loading.value = isLoading;
	};

	const reset = () => {
		stats.value = null;
		controllers.value = [];
		brightness.value = DEFAULT_LED_CONFIG.defaultBrightness;
		loading.value = false;
	};

	return {
		// State
		stats,
		controllers,
		brightness,
		loading,

		// Getters
		isRunning,
		currentMode,
		frameSize,
		matrixSize,
		controllerCount,
		connectedControllers,
		isHealthy,

		// Actions
		setStats,
		setControllers,
		updateController,
		setBrightness,
		setRunning,
		setMode,
		setLoading,
		reset,
	};
});
