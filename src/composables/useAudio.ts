import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { onMounted, onUnmounted } from 'vue';
import { useAudioStore } from '../stores/audio';
import { useLogsStore } from '../stores/logs';
import type { ActionResult } from '../types';

export function useAudio() {
	const audioStore = useAudioStore();
	const logsStore = useLogsStore();

	let unlistenSpectrum: UnlistenFn | null = null;
	let unlistenAudioStatus: UnlistenFn | null = null;

	// Récupérer les périphériques audio disponibles
	const getAudioDevices = async (): Promise<ActionResult> => {
		audioStore.setLoading(true);
		try {
			const devices = await invoke<string[]>('get_audio_devices');
			audioStore.setDevices(devices);
			logsStore.addLog(`🎤 Found ${devices.length} audio devices`, 'success', 'audio');
			return { success: true, message: `Found ${devices.length} audio devices` };
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			audioStore.setError(errorMessage);
			logsStore.addLog(`Failed to get audio devices: ${errorMessage}`, 'error', 'audio');
			return { success: false, message: `Failed to get audio devices: ${errorMessage}` };
		} finally {
			audioStore.setLoading(false);
		}
	};

	// Démarrer la capture audio
	const startAudioCapture = async (): Promise<ActionResult> => {
		if (audioStore.state.isCapturing) {
			return { success: false, message: 'Audio capture already running' };
		}

		audioStore.setLoading(true);
		try {
			const result = await invoke<string>('start_audio_capture');
			audioStore.setCapturing(true);
			audioStore.clearError();
			logsStore.addLog('🎧 Audio capture started', 'success', 'audio');
			return { success: true, message: result };
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			audioStore.setError(errorMessage);
			logsStore.addLog(`Failed to start audio capture: ${errorMessage}`, 'error', 'audio');
			return { success: false, message: `Failed to start audio capture: ${errorMessage}` };
		} finally {
			audioStore.setLoading(false);
		}
	};

	// Arrêter la capture audio
	const stopAudioCapture = async (): Promise<ActionResult> => {
		audioStore.setLoading(true);
		try {
			const result = await invoke<string>('stop_audio_capture');
			audioStore.setCapturing(false);
			logsStore.addLog('🛑 Audio capture stopped', 'info', 'audio');
			return { success: true, message: result };
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			logsStore.addLog(`Failed to stop audio capture: ${errorMessage}`, 'error', 'audio');
			return { success: false, message: `Failed to stop audio capture: ${errorMessage}` };
		} finally {
			audioStore.setLoading(false);
		}
	};

	// Définir le gain audio
	const setAudioGain = async (gain: number): Promise<ActionResult> => {
		try {
			const result = await invoke<string>('set_audio_gain', { gain });
			audioStore.setGain(gain);
			logsStore.addLog(`🔊 Audio gain set to ${gain.toFixed(1)}x`, 'info', 'audio');
			return { success: true, message: result };
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			logsStore.addLog(`Failed to set audio gain: ${errorMessage}`, 'error', 'audio');
			return { success: false, message: `Failed to set audio gain: ${errorMessage}` };
		}
	};

	// Récupérer le gain audio actuel
	const getAudioGain = async (): Promise<number> => {
		try {
			const gain = await invoke<number>('get_audio_gain');
			audioStore.setGain(gain);
			return gain;
		} catch (error) {
			logsStore.addLog('Failed to get audio gain', 'warning', 'audio');
			return audioStore.state.currentGain;
		}
	};

	// Récupérer le spectre actuel
	const getCurrentSpectrum = async (): Promise<number[]> => {
		try {
			const spectrum = await invoke<number[]>('get_current_spectrum');
			audioStore.updateSpectrum(spectrum);
			return spectrum;
		} catch (error) {
			logsStore.addLog('Failed to get current spectrum', 'warning', 'audio');
			return [];
		}
	};

	// Gestionnaire des données de spectre
	const handleSpectrumData = (spectrum: number[]): void => {
		audioStore.updateSpectrum(spectrum);
	};

	// Gestionnaire du statut audio
	const handleAudioStatus = (status: any): void => {
		logsStore.addLog(`📊 Audio status: ${status.status}`, 'info', 'audio');

		if (status.status === 'started') {
			audioStore.setCapturing(true);
			audioStore.clearError();
		} else if (status.status === 'stopped' || status.status === 'error') {
			audioStore.setCapturing(false);
			if (status.status === 'error') {
				audioStore.setError(status.message);
			}
		}
	};

	// Configuration des écouteurs d'événements
	const setupEventListeners = async (): Promise<void> => {
		try {
			unlistenSpectrum = await listen<number[]>('spectrum_data', (event) => {
				handleSpectrumData(event.payload);
			});

			unlistenAudioStatus = await listen<any>('audio_status', (event) => {
				handleAudioStatus(event.payload);
			});

			logsStore.addLog('✅ Audio event listeners ready', 'success', 'audio');
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			audioStore.setError(`Failed to setup event listeners: ${errorMessage}`);
			logsStore.addLog(`❌ Error setting up audio event listeners: ${errorMessage}`, 'error', 'audio');
		}
	};

	// Nettoyage
	const cleanup = (): void => {
		if (unlistenSpectrum) {
			unlistenSpectrum();
			unlistenSpectrum = null;
		}
		if (unlistenAudioStatus) {
			unlistenAudioStatus();
			unlistenAudioStatus = null;
		}
	};

	// Lifecycle
	onMounted(() => {
		logsStore.addLog('🎧 Audio composable mounted', 'debug', 'audio');
		setupEventListeners();
		getAudioDevices();
		getAudioGain();
	});

	onUnmounted(() => {
		logsStore.addLog('💀 Audio composable unmounting', 'debug', 'audio');
		cleanup();
		if (audioStore.state.isCapturing) {
			stopAudioCapture();
		}
	});

	return {
		// Store state access
		state: audioStore.state,
		loading: audioStore.loading,
		isHealthy: audioStore.isHealthy,
		spectrumPeak: audioStore.spectrumPeak,
		spectrumRMS: audioStore.spectrumRMS,
		selectedDevice: audioStore.selectedDevice,

		// Actions
		getAudioDevices,
		startAudioCapture,
		stopAudioCapture,
		setAudioGain,
		getAudioGain,
		getCurrentSpectrum,
		cleanup,
		reset: audioStore.reset,
	};
}
