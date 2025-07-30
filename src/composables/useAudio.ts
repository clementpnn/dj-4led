// composables/useAudio.ts
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { onMounted, onUnmounted } from 'vue';
import { useAudioStore } from '../stores/audio';
import type { ActionResult } from '../types';

export function useAudio() {
	// Store instance
	const audioStore = useAudioStore();

	// Event listeners references
	let unlistenSpectrum: UnlistenFn | null = null;
	let unlistenAudioStatus: UnlistenFn | null = null;
	let unlistenGainChanged: UnlistenFn | null = null;

	// ===== AUDIO CAPTURE ACTIONS =====

	const startCapture = async (): Promise<ActionResult> => {
		if (audioStore.state.isCapturing) {
			return { success: false, message: 'Audio capture already running' };
		}

		audioStore.setLoading(true);
		audioStore.clearError();

		try {
			const result = await invoke<any>('audio_start_capture');
			console.log('🎧 Audio capture started:', result);

			return {
				success: true,
				message: result.message || 'Audio capture started',
				data: result,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			audioStore.setError(errorMessage);
			console.error('❌ Failed to start audio:', errorMessage);
			return { success: false, message: errorMessage };
		} finally {
			audioStore.setLoading(false);
		}
	};

	const stopCapture = async (): Promise<ActionResult> => {
		audioStore.setLoading(true);

		try {
			const result = await invoke<any>('audio_stop_capture');
			audioStore.setCapturing(false);
			console.log('🛑 Audio capture stopped:', result);

			return {
				success: true,
				message: result.message || 'Audio capture stopped',
				data: result,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			audioStore.setError(errorMessage);
			console.error('❌ Failed to stop audio:', errorMessage);
			return { success: false, message: errorMessage };
		} finally {
			audioStore.setLoading(false);
		}
	};

	// ===== GAIN MANAGEMENT =====

	const setGain = async (newGain: number): Promise<ActionResult> => {
		if (newGain < 0.1 || newGain > 5.0) {
			return { success: false, message: 'Gain must be between 0.1 and 5.0' };
		}

		try {
			const result = await invoke<any>('audio_set_gain', { gain: newGain });
			audioStore.setGain(newGain);
			console.log(`🔊 Audio gain set to ${newGain}:`, result);

			return {
				success: true,
				message: result.message || `Gain set to ${newGain}`,
				data: result,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			audioStore.setError(errorMessage);
			console.error('❌ Failed to set gain:', errorMessage);
			return { success: false, message: errorMessage };
		}
	};

	const getGain = async (): Promise<ActionResult> => {
		try {
			const currentGain = await invoke<number>('audio_get_gain');
			audioStore.setGain(currentGain);

			return {
				success: true,
				message: 'Gain retrieved successfully',
				data: currentGain,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			console.warn('Failed to get audio gain:', errorMessage);
			return {
				success: false,
				message: errorMessage,
				data: audioStore.state.currentGain,
			};
		}
	};

	// ===== DEVICE MANAGEMENT =====

	const getDevices = async (): Promise<ActionResult> => {
		audioStore.setLoading(true);

		try {
			const result = await invoke<any>('audio_get_devices');
			const devices = result.devices || [];

			// Extract device names for the store
			const deviceNames = devices.map((device: any) => device.name);
			audioStore.setDevices(deviceNames);

			console.log(`🎤 Found ${devices.length} audio devices:`, result);

			return {
				success: true,
				message: `Found ${devices.length} audio devices`,
				data: devices,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			audioStore.setError(errorMessage);
			console.error('❌ Failed to get audio devices:', errorMessage);
			return { success: false, message: errorMessage };
		} finally {
			audioStore.setLoading(false);
		}
	};

	// ===== STATUS & DIAGNOSTICS =====

	const getStatus = async (): Promise<ActionResult> => {
		try {
			const status = await invoke<any>('audio_get_status');

			// Update store with backend status
			audioStore.setCapturing(status.running);
			audioStore.setGain(status.gain);

			console.log('📊 Audio status:', status);
			return { success: true, message: 'Status retrieved', data: status };
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			console.warn('Failed to get audio status:', errorMessage);
			return { success: false, message: errorMessage };
		}
	};

	const getSpectrum = async (): Promise<ActionResult> => {
		try {
			const result = await invoke<any>('audio_get_spectrum');

			if (result.spectrum && Array.isArray(result.spectrum)) {
				audioStore.updateSpectrum(result.spectrum);
			}

			return { success: true, message: 'Spectrum retrieved', data: result };
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			console.warn('Failed to get spectrum:', errorMessage);
			return { success: false, message: errorMessage };
		}
	};

	const testInput = async (): Promise<ActionResult> => {
		try {
			const result = await invoke<any>('audio_test_input');
			console.log('🔍 Audio input test:', result);

			return {
				success: true,
				message: result.message || 'Audio test completed',
				data: result,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			audioStore.setError(errorMessage);
			console.error('❌ Audio test failed:', errorMessage);
			return { success: false, message: errorMessage };
		}
	};

	// ===== EVENT HANDLERS =====

	const handleSpectrumUpdate = (event: any) => {
		const data = event.payload;

		// Validate payload structure
		if (data && Array.isArray(data.spectrum)) {
			audioStore.updateSpectrum(data.spectrum);
		} else if (Array.isArray(data)) {
			// Fallback: direct array
			audioStore.updateSpectrum(data);
		} else {
			console.warn('🔊 Invalid spectrum data received:', data);
		}
	};

	const handleAudioStatus = (event: any) => {
		const status = event.payload;
		console.log('📊 Audio status event:', status);

		if (!status || typeof status.status !== 'string') {
			console.warn('Invalid audio status event:', status);
			return;
		}

		switch (status.status) {
			case 'started':
				audioStore.setCapturing(true);
				audioStore.clearError();
				break;

			case 'stopped':
				audioStore.setCapturing(false);
				audioStore.updateSpectrum([]); // Clear spectrum
				break;

			case 'error':
				audioStore.setCapturing(false);
				audioStore.updateSpectrum([]); // Clear spectrum
				audioStore.setError(status.message || 'Audio error occurred');
				break;

			default:
				console.warn('Unknown audio status:', status.status);
		}
	};

	const handleGainChanged = (event: any) => {
		const data = event.payload;

		if (data && typeof data.gain === 'number') {
			audioStore.setGain(data.gain);
			console.log(`🔊 Gain changed to ${data.gain}`);
		} else {
			console.warn('Invalid gain change event:', data);
		}
	};

	// ===== EVENT LISTENERS SETUP =====

	const setupListeners = async (): Promise<void> => {
		try {
			unlistenSpectrum = await listen('spectrum_update', handleSpectrumUpdate);
			unlistenAudioStatus = await listen('audio_status', handleAudioStatus);
			unlistenGainChanged = await listen('audio_gain_changed', handleGainChanged);

			console.log('✅ Audio event listeners setup complete');
		} catch (err) {
			console.error('❌ Failed to setup audio event listeners:', err);
			audioStore.setError('Failed to setup event listeners');
		}
	};

	const cleanup = (): void => {
		const listeners = [
			{ fn: unlistenSpectrum, name: 'spectrum_update' },
			{ fn: unlistenAudioStatus, name: 'audio_status' },
			{ fn: unlistenGainChanged, name: 'audio_gain_changed' },
		];

		listeners.forEach(({ fn, name }) => {
			if (fn) {
				try {
					fn();
					console.log(`✅ Cleaned up ${name} listener`);
				} catch (err) {
					console.warn(`❌ Error cleaning up ${name} listener:`, err);
				}
			}
		});

		// Reset listener references
		unlistenSpectrum = null;
		unlistenAudioStatus = null;
		unlistenGainChanged = null;
	};

	// ===== INITIALIZATION =====

	const initialize = async (): Promise<void> => {
		console.log('🎧 Initializing audio composable...');

		try {
			await setupListeners();
			await getDevices();
			await getStatus();
			await getGain();
			audioStore.setCapturing(false);
			audioStore.updateSpectrum([]);

			console.log('✅ Audio composable initialized successfully');
		} catch (err) {
			console.error('❌ Failed to initialize audio composable:', err);
			audioStore.setError('Failed to initialize audio system');
		}
	};

	// ===== LIFECYCLE =====

	onMounted(() => {
		console.log('🎧 Audio composable mounted');
		initialize();
	});

	onUnmounted(() => {
		console.log('💀 Audio composable unmounting');
		cleanup();

		// Stop capture if running
		if (audioStore.state.isCapturing) {
			stopCapture().catch(console.error);
		}
	});

	// ===== PUBLIC API =====

	return {
		...audioStore,

		// Actions
		startCapture,
		stopCapture,
		setGain,
		getGain,
		getDevices,
		getStatus,
		getSpectrum,
		testInput,

		// Utilities
		initialize,
		cleanup,
	};
}
