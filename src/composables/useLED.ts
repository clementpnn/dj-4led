// composables/useLED.ts
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { onMounted, onUnmounted, ref } from 'vue';
import { useLEDStore } from '../stores/led';
import type { ActionResult, LEDController, LEDStats } from '../types';

export function useLED() {
	// Store instance
	const ledStore = useLEDStore();

	// Local state for non-store data
	const error = ref<string | null>(null);
	const testPatterns = ref<any[]>([]);

	// Event listeners references
	let unlistenLedStatus: UnlistenFn | null = null;
	let unlistenLedStats: UnlistenFn | null = null;
	let unlistenBrightnessChanged: UnlistenFn | null = null;
	let unlistenTestStarted: UnlistenFn | null = null;
	let unlistenTestCompleted: UnlistenFn | null = null;
	let unlistenConnectivityTest: UnlistenFn | null = null;

	// ===== OUTPUT CONTROL ACTIONS =====

	const startOutput = async (outputMode: 'simulator' | 'production' = 'simulator'): Promise<ActionResult> => {
		if (ledStore.isRunning) {
			return { success: false, message: 'LED output already running' };
		}

		ledStore.setLoading(true);
		error.value = null;

		try {
			const result = await invoke<any>('led_start_output', { mode: outputMode });

			// Update store with new mode
			ledStore.setMode(outputMode);

			console.log(`💡 LED output started in ${outputMode} mode:`, result);
			return {
				success: true,
				message: result.message || `LED output started in ${outputMode} mode`,
				data: result,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			error.value = errorMessage;
			console.error('❌ Failed to start LED output:', errorMessage);
			return { success: false, message: errorMessage };
		} finally {
			ledStore.setLoading(false);
		}
	};

	const stopOutput = async (): Promise<ActionResult> => {
		ledStore.setLoading(true);

		try {
			const result = await invoke<any>('led_stop_output');
			ledStore.setRunning(false);

			console.log('🛑 LED output stopped:', result);
			return {
				success: true,
				message: result.message || 'LED output stopped',
				data: result,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			error.value = errorMessage;
			console.error('❌ Failed to stop LED output:', errorMessage);
			return { success: false, message: errorMessage };
		} finally {
			ledStore.setLoading(false);
		}
	};

	// ===== BRIGHTNESS CONTROL =====

	const setBrightness = async (newBrightness: number): Promise<ActionResult> => {
		if (newBrightness < 0 || newBrightness > 1) {
			return { success: false, message: 'Brightness must be between 0 and 1' };
		}

		try {
			const result = await invoke<any>('led_set_brightness', { brightness: newBrightness });
			ledStore.setBrightness(newBrightness);

			console.log(`🔆 LED brightness set to ${Math.round(newBrightness * 100)}%:`, result);
			return {
				success: true,
				message: result.message || `Brightness set to ${Math.round(newBrightness * 100)}%`,
				data: result,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			error.value = errorMessage;
			console.error('❌ Failed to set LED brightness:', errorMessage);
			return { success: false, message: errorMessage };
		}
	};

	const getBrightness = async (): Promise<ActionResult> => {
		try {
			const result = await invoke<any>('led_get_brightness');
			ledStore.setBrightness(result.brightness);

			console.log('💡 Current LED brightness:', result);
			return {
				success: true,
				message: 'Brightness retrieved',
				data: result,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			console.warn('Failed to get LED brightness:', errorMessage);
			return { success: false, message: errorMessage };
		}
	};

	// ===== STATUS & INFO ACTIONS =====

	const getStatus = async (): Promise<ActionResult> => {
		try {
			const result = await invoke<any>('led_get_status');

			// Transform to LEDStats format for store
			const stats: LEDStats = {
				is_running: result.running,
				mode: result.mode,
				brightness: result.brightness,
				frame_size: result.frame_size,
				matrix_size: result.matrix_size,
				target_fps: result.target_fps,
				frame_time_ms: result.frame_time_ms,
				controllers: 0, // Will be updated by getControllers
			};

			ledStore.setStats(stats);

			console.log('📊 LED status:', result);
			return {
				success: true,
				message: 'Status retrieved',
				data: result,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			console.warn('Failed to get LED status:', errorMessage);
			return { success: false, message: errorMessage };
		}
	};

	const getControllers = async (): Promise<ActionResult> => {
		try {
			const result = await invoke<any>('led_get_controllers');

			// Transform to LEDController format
			const controllers: LEDController[] = (result.controllers || []).map((ctrl: any) => ({
				id: ctrl.id,
				name: ctrl.name || ctrl.id,
				ip: ctrl.ip,
				status: ctrl.enabled ? 'connected' : 'disconnected',
				type: result.mode === 'production' ? 'hardware' : 'simulator',
				lastSeen: Date.now(),
			}));

			ledStore.setControllers(controllers);

			// Update controller count in stats
			if (ledStore.stats) {
				ledStore.setStats({
					...ledStore.stats,
					controllers: controllers.length,
				});
			}

			console.log(`📋 LED controllers (${result.mode} mode):`, result);
			return {
				success: true,
				message: `Found ${controllers.length} controllers`,
				data: result,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			console.warn('Failed to get LED controllers:', errorMessage);
			return { success: false, message: errorMessage };
		}
	};

	// ===== TEST FUNCTIONS =====

	const sendTestPattern = async (pattern: string, durationMs: number = 3000): Promise<ActionResult> => {
		try {
			const result = await invoke<any>('led_send_test_pattern', {
				pattern,
				durationMs,
			});

			console.log(`🎨 LED test pattern "${pattern}" sent:`, result);
			return {
				success: true,
				message: result.message || `Test pattern "${pattern}" sent`,
				data: result,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			error.value = errorMessage;
			console.error(`❌ Failed to send test pattern "${pattern}":`, errorMessage);
			return { success: false, message: errorMessage };
		}
	};

	const testConnectivity = async (): Promise<ActionResult> => {
		ledStore.setLoading(true);

		try {
			const result = await invoke<any>('led_test_connectivity');

			// Update controller statuses based on test results
			if (result.results) {
				Object.entries(result.results).forEach(([controllerId, isWorking]) => {
					ledStore.updateController(controllerId, {
						status: isWorking ? 'connected' : 'error',
						lastSeen: Date.now(),
					});
				});
			}

			console.log('🔍 LED connectivity test:', result);
			return {
				success: true,
				message: `Connectivity test: ${result.active_controllers}/${result.total_controllers} controllers OK`,
				data: result,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			error.value = errorMessage;
			console.error('❌ LED connectivity test failed:', errorMessage);
			return { success: false, message: errorMessage };
		} finally {
			ledStore.setLoading(false);
		}
	};

	const getTestPatterns = async (): Promise<ActionResult> => {
		try {
			const result = await invoke<any>('led_get_test_patterns');
			testPatterns.value = [...(result.basic || []), ...(result.patterns || [])];

			console.log('🎨 LED test patterns:', result);
			return {
				success: true,
				message: `Found ${testPatterns.value.length} test patterns`,
				data: result,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			console.warn('Failed to get test patterns:', errorMessage);
			return { success: false, message: errorMessage };
		}
	};

	// ===== UTILITY ACTIONS =====

	const clearDisplay = async (): Promise<ActionResult> => {
		try {
			const result = await invoke<any>('led_clear_display');

			console.log('🧹 LED display cleared:', result);
			return {
				success: true,
				message: result.message || 'Display cleared',
				data: result,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			error.value = errorMessage;
			console.error('❌ Failed to clear LED display:', errorMessage);
			return { success: false, message: errorMessage };
		}
	};

	const getFrameData = async (): Promise<ActionResult> => {
		try {
			const result = await invoke<any>('led_get_frame_data');

			console.log('🖼️ LED frame data:', result);
			return {
				success: true,
				message: 'Frame data retrieved',
				data: result,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			console.warn('Failed to get frame data:', errorMessage);
			return { success: false, message: errorMessage };
		}
	};

	// ===== EVENT HANDLERS =====

	const handleLedStatus = (event: any) => {
		const status = event.payload;
		console.log('📊 LED status event:', status);

		switch (status.status) {
			case 'started':
				ledStore.setRunning(true);
				if (status.mode) {
					ledStore.setMode(status.mode);
				}
				error.value = null;
				break;

			case 'stopped':
			case 'stopping':
				ledStore.setRunning(false);
				break;

			case 'error':
				ledStore.setRunning(false);
				error.value = status.error || 'LED error occurred';
				break;
		}
	};

	const handleLedStats = (event: any) => {
		const statsData = event.payload;

		// Update store stats with new data
		if (ledStore.stats) {
			ledStore.setStats({
				...ledStore.stats,
				// Update any relevant stats from the event
			});
		}

		// Log only significant changes to avoid spam
		if (statsData.frame_count && statsData.frame_count % 1000 === 0) {
			console.log('📊 LED stats update:', statsData);
		}
	};

	const handleBrightnessChanged = (event: any) => {
		const data = event.payload;
		if (typeof data.brightness === 'number') {
			ledStore.setBrightness(data.brightness);
			console.log(`🔆 Brightness changed to ${Math.round(data.brightness * 100)}%`);
		}
	};

	const handleTestStarted = (event: any) => {
		const data = event.payload;
		console.log(`🎨 Test pattern "${data.pattern}" started for ${data.duration_ms}ms`);
	};

	const handleTestCompleted = (event: any) => {
		const data = event.payload;
		console.log(`✅ Test pattern "${data.pattern}" completed`);
	};

	const handleConnectivityTest = (event: any) => {
		const data = event.payload;
		console.log('🔍 Connectivity test result:', data);

		// Update controller statuses if results provided
		if (data.results) {
			Object.entries(data.results).forEach(([controllerId, isWorking]) => {
				ledStore.updateController(controllerId, {
					status: isWorking ? 'connected' : 'error',
					lastSeen: Date.now(),
				});
			});
		}
	};

	// ===== EVENT LISTENERS SETUP =====

	const setupListeners = async (): Promise<void> => {
		try {
			unlistenLedStatus = await listen('led_status', handleLedStatus);
			unlistenLedStats = await listen('led_stats', handleLedStats);
			unlistenBrightnessChanged = await listen('led_brightness_changed', handleBrightnessChanged);
			unlistenTestStarted = await listen('led_test_started', handleTestStarted);
			unlistenTestCompleted = await listen('led_test_completed', handleTestCompleted);
			unlistenConnectivityTest = await listen('led_connectivity_test_completed', handleConnectivityTest);

			console.log('✅ LED event listeners setup complete');
		} catch (err) {
			console.error('❌ Failed to setup LED event listeners:', err);
			error.value = 'Failed to setup event listeners';
		}
	};

	const cleanup = (): void => {
		const listeners = [
			{ fn: unlistenLedStatus, name: 'led_status' },
			{ fn: unlistenLedStats, name: 'led_stats' },
			{ fn: unlistenBrightnessChanged, name: 'led_brightness_changed' },
			{ fn: unlistenTestStarted, name: 'led_test_started' },
			{ fn: unlistenTestCompleted, name: 'led_test_completed' },
			{ fn: unlistenConnectivityTest, name: 'led_connectivity_test_completed' },
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

		unlistenLedStatus = null;
		unlistenLedStats = null;
		unlistenBrightnessChanged = null;
		unlistenTestStarted = null;
		unlistenTestCompleted = null;
		unlistenConnectivityTest = null;
	};

	// ===== INITIALIZATION =====

	const initialize = async (): Promise<void> => {
		console.log('💡 Initializing LED composable...');

		try {
			await setupListeners();
			await getStatus();
			await getBrightness();
			await getControllers();
			await getTestPatterns();

			console.log('✅ LED composable initialized successfully');
		} catch (err) {
			console.error('❌ Failed to initialize LED composable:', err);
			error.value = 'Failed to initialize LED system';
		}
	};

	// ===== LIFECYCLE =====

	onMounted(() => {
		console.log('💡 LED composable mounted');
		initialize();
	});

	onUnmounted(() => {
		console.log('💀 LED composable unmounting');
		cleanup();
	});

	// ===== PUBLIC API =====

	return {
		// Store access
		...ledStore,

		// Local state
		error,
		testPatterns,

		// Actions
		startOutput,
		stopOutput,
		setBrightness,
		getBrightness,
		getStatus,
		getControllers,
		sendTestPattern,
		testConnectivity,
		getTestPatterns,
		clearDisplay,
		getFrameData,

		// Utilities
		initialize,
		cleanup,
	};
}
