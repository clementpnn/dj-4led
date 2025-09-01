// composables/useApp.ts
import { computed, ref } from 'vue';

import { useAudio } from '@/composables/useAudio';
import { useColors } from '@/composables/useColors';
import { useEffects } from '@/composables/useEffects';
import { useFrames } from '@/composables/useFrames';
import { useLED } from '@/composables/useLED';
import { useLogs } from '@/composables/useLogs';
import { usePresets } from '@/composables/usePresets';
import { useSystem } from '@/composables/useSystem';
import type { ActionResult } from '@/types';

export const useApp = () => {
	// Initialize all composables
	const audio = useAudio();
	const effects = useEffects();
	const colors = useColors();
	const led = useLED();
	const frames = useFrames();
	const system = useSystem();
	const presets = usePresets();
	const logs = useLogs();

	// App state
	const isInitialized = ref(false);
	const isShuttingDown = ref(false);

	// Computed app health
	const appHealth = computed(() => {
		const healthyComponents = [audio.state.isCapturing, led.isRunning, system.isHealthy].filter(Boolean).length;

		if (healthyComponents >= 3) return 'healthy';
		if (healthyComponents >= 1) return 'warning';
		return 'critical';
	});

	// Simple initialization
	const initializeApp = async (): Promise<ActionResult> => {
		if (isInitialized.value) {
			return { success: true, message: 'Application already initialized' };
		}

		try {
			logs.log('🚀 Initializing DJ-4LED...', 'info', 'system');

			// Load initial data
			await Promise.allSettled([
				audio.getDevices(),
				effects.getEffectsList(),
				colors.getColorMode(),
				led.getStatus(),
				system.getStatus(),
			]);

			isInitialized.value = true;
			logs.log('✅ DJ-4LED initialized successfully', 'success', 'system');

			return { success: true, message: 'Application initialized successfully' };
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			logs.log(`❌ Initialization failed: ${errorMessage}`, 'error', 'system');
			return { success: false, message: `Initialization failed: ${errorMessage}` };
		}
	};

	// Simple shutdown
	const shutdownApp = async (): Promise<ActionResult> => {
		if (isShuttingDown.value) {
			return { success: true, message: 'Shutdown already in progress' };
		}

		try {
			isShuttingDown.value = true;
			logs.log('🛑 Shutting down DJ-4LED...', 'info', 'system');

			// Stop services
			if (audio.state.isCapturing) {
				await audio.stopCapture();
			}

			if (led.isRunning) {
				await led.stopOutput();
			}

			isInitialized.value = false;
			logs.log('✅ DJ-4LED shut down successfully', 'success', 'system');

			return { success: true, message: 'Application shut down successfully' };
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			logs.log(`❌ Shutdown failed: ${errorMessage}`, 'error', 'system');
			return { success: false, message: `Shutdown failed: ${errorMessage}` };
		} finally {
			isShuttingDown.value = false;
		}
	};

	// Quick start - Audio + LED
	const quickStart = async (mode: 'simulator' | 'production' = 'simulator'): Promise<ActionResult> => {
		try {
			logs.log('⚡ Quick start initiated...', 'info', 'user');

			// Start audio
			const audioResult = await audio.startCapture();
			if (!audioResult.success) {
				logs.log(`Audio start failed: ${audioResult.message}`, 'error', 'audio');
			}

			// Start LED
			const ledResult = await led.startOutput(mode);
			if (!ledResult.success) {
				logs.log(`LED start failed: ${ledResult.message}`, 'error', 'led');
			}

			// Set default effect if we have effects
			if (effects.availableEffects.length > 0) {
				await effects.setEffect(effects.availableEffects[0].id);
			}

			const success = audioResult.success && ledResult.success;
			const message = success ? 'Quick start completed successfully' : 'Quick start completed with issues';
			const logType = success ? 'success' : 'warning';

			logs.log(`⚡ ${message}`, logType, 'user');
			return { success, message };
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			logs.log(`⚡ Quick start failed: ${errorMessage}`, 'error', 'user');
			return { success: false, message: `Quick start failed: ${errorMessage}` };
		}
	};

	// Quick stop
	const quickStop = async (): Promise<ActionResult> => {
		try {
			logs.log('🛑 Quick stop initiated...', 'info', 'user');

			const tasks = [];

			if (audio.state.isCapturing) {
				tasks.push(audio.stopCapture());
			}

			if (led.isRunning) {
				tasks.push(led.stopOutput());
			}

			await Promise.allSettled(tasks);

			logs.log('🛑 Quick stop completed', 'success', 'user');
			return { success: true, message: 'Quick stop completed' };
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			logs.log(`🛑 Quick stop failed: ${errorMessage}`, 'error', 'user');
			return { success: false, message: `Quick stop failed: ${errorMessage}` };
		}
	};

	// Apply preset with logging
	const applyPreset = async (presetId: string): Promise<ActionResult> => {
		try {
			const composables = { audio, effects, colors, led };
			const result = await presets.applyPreset(presetId, composables);
			logs.log(`Preset applied: ${result.message}`, result.success ? 'success' : 'error', 'user');
			return result;
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			logs.log(`Preset application failed: ${errorMessage}`, 'error', 'user');
			return { success: false, message: errorMessage };
		}
	};

	// Save current as preset
	const saveCurrentAsPreset = async (name: string, description = ''): Promise<ActionResult> => {
		try {
			const composables = { audio, effects, colors, led };
			const config = await presets.captureCurrentConfig(composables);
			const result = await presets.createPreset(name, description, config);
			logs.log(`Preset saved: ${name}`, result.success ? 'success' : 'error', 'user');
			return result;
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			logs.log(`Preset save failed: ${errorMessage}`, 'error', 'user');
			return { success: false, message: errorMessage };
		}
	};

	// Get app status
	const getAppStatus = () => {
		return {
			initialized: isInitialized.value,
			shuttingDown: isShuttingDown.value,
			health: appHealth.value,
			components: {
				audio: {
					running: audio.state.isCapturing,
					devices: audio.state.devices.length,
				},
				led: {
					running: led.isRunning,
					mode: led.currentMode,
					brightness: led.brightness,
				},
				effects: {
					current: effects.currentEffectName,
					transitioning: effects.isTransitioning,
					total: effects.availableEffects.length,
				},
				frames: {
					fps: frames.stats.fps,
					health: frames.healthStatus.value,
				},
			},
		};
	};

	// Reset all to defaults
	const resetAll = async (): Promise<ActionResult> => {
		try {
			logs.log('🔄 Resetting all systems...', 'info', 'system');

			// Stop everything first
			await quickStop();

			// Reset stores
			audio.reset();
			effects.reset();
			colors.reset();
			led.reset();
			frames.reset();
			system.reset();

			logs.log('🔄 All systems reset to defaults', 'success', 'system');
			return { success: true, message: 'All systems reset successfully' };
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			logs.log(`🔄 Reset failed: ${errorMessage}`, 'error', 'system');
			return { success: false, message: `Reset failed: ${errorMessage}` };
		}
	};

	return {
		// Composables
		audio,
		effects,
		colors,
		led,
		frames,
		system,
		presets,
		logs,

		// App state
		isInitialized,
		isShuttingDown,
		appHealth,

		// App actions
		initializeApp,
		shutdownApp,
		quickStart,
		quickStop,
		applyPreset,
		saveCurrentAsPreset,
		getAppStatus,
		resetAll,
	};
};
