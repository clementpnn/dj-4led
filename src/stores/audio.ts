import { defineStore } from 'pinia';
import { computed, readonly, ref } from 'vue';
import { DEFAULT_AUDIO_CONFIG } from '../config';
import type { AudioState, AudioStats } from '../types';

export const useAudioStore = defineStore('audio', () => {
	// ===== STATE =====
	const state = ref<AudioState>({
		isCapturing: false,
		devices: [],
		currentGain: DEFAULT_AUDIO_CONFIG.defaultGain,
		spectrum: [],
		error: null,
	});

	const stats = ref<AudioStats>({
		sampleRate: 44100,
		channels: 2,
		bufferSize: 1024,
		inputLatency: 0,
		outputLatency: 0,
	});

	const loading = ref(false);
	const selectedDeviceIndex = ref<number | null>(null);

	// ===== GETTERS =====
	const isHealthy = computed(() => state.value.isCapturing && state.value.spectrum.length > 0 && !state.value.error);

	const spectrumPeak = computed(() => {
		if (!state.value.isCapturing || state.value.spectrum.length === 0) return 0;
		const peak = Math.max(...state.value.spectrum);
		return Math.min(1.0, peak);
	});

	const spectrumRMS = computed(() => {
		if (!state.value.isCapturing || state.value.spectrum.length === 0) return 0;
		const sum = state.value.spectrum.reduce((acc, val) => acc + val * val, 0);
		const rms = Math.sqrt(sum / state.value.spectrum.length);
		return Math.min(1.0, rms);
	});

	const selectedDevice = computed(() =>
		selectedDeviceIndex.value !== null && state.value.devices[selectedDeviceIndex.value]
			? state.value.devices[selectedDeviceIndex.value]
			: 'Auto-detect'
	);

	// ===== ACTIONS =====
	const updateState = (newState: Partial<AudioState>) => {
		Object.assign(state.value, newState);
	};

	const updateStats = (newStats: Partial<AudioStats>) => {
		Object.assign(stats.value, newStats);
	};

	const setLoading = (isLoading: boolean) => {
		loading.value = isLoading;
	};

	const setError = (error: string | null) => {
		state.value.error = error;
	};

	const clearError = () => {
		state.value.error = null;
	};

	const setDevices = (devices: string[]) => {
		state.value.devices = devices;
	};

	const setSelectedDevice = (deviceIndex: number | null) => {
		selectedDeviceIndex.value = deviceIndex;
	};

	const setCapturing = (isCapturing: boolean) => {
		state.value.isCapturing = isCapturing;
		if (!isCapturing) {
			state.value.spectrum = [];
		}
	};

	const setGain = (gain: number) => {
		const clampedGain = Math.max(DEFAULT_AUDIO_CONFIG.minGain, Math.min(DEFAULT_AUDIO_CONFIG.maxGain, gain));
		state.value.currentGain = clampedGain;
	};

	const updateSpectrum = (spectrum: number[]) => {
		if (state.value.isCapturing && spectrum?.length > 0) {
			state.value.spectrum = spectrum.slice(0, DEFAULT_AUDIO_CONFIG.spectrumBands);
		} else if (!state.value.isCapturing) {
			state.value.spectrum = [];
		}
	};

	const reset = () => {
		state.value = {
			isCapturing: false,
			devices: [],
			currentGain: DEFAULT_AUDIO_CONFIG.defaultGain,
			spectrum: [],
			error: null,
		};

		stats.value = {
			sampleRate: 44100,
			channels: 2,
			bufferSize: 1024,
			inputLatency: 0,
			outputLatency: 0,
		};

		loading.value = false;
		selectedDeviceIndex.value = null;
	};

	return {
		// State
		state: readonly(state),
		stats: readonly(stats),
		loading: readonly(loading),
		selectedDeviceIndex: readonly(selectedDeviceIndex),

		// Getters
		isHealthy,
		spectrumPeak,
		spectrumRMS,
		selectedDevice,

		// Actions
		updateState,
		updateStats,
		setLoading,
		setError,
		clearError,
		setDevices,
		setSelectedDevice,
		setCapturing,
		setGain,
		updateSpectrum,
		reset,
	};
});
