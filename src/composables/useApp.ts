import type { ActionResult } from '../types';
import { useAudio } from './useAudio';
import { useColors } from './useColors';
import { useEffects } from './useEffects';
import { useFrames } from './useFrames';
import { useLED } from './useLED';
import { useLogs } from './useLogs';
import { usePresets } from './usePresets';
import { useSystem } from './useSystem';

export const useApp = () => {
	const audio = useAudio();
	const effects = useEffects();
	const colors = useColors();
	const led = useLED();
	const frames = useFrames();
	const system = useSystem();
	const presets = usePresets();
	const logs = useLogs();

	// Initialisation simple
	const initializeApp = async (): Promise<ActionResult> => {
		try {
			logs.initLogs();
			logs.log('🚀 Initializing DJ-4LED...', 'info', 'system');

			// Charger les presets
			presets.initializeFromStorage();

			// Charger les données initiales
			await Promise.allSettled([
				audio.getAudioDevices(),
				effects.getAvailableEffects(),
				colors.getColorMode(),
				led.getLEDStats(),
				system.getSystemStats(),
			]);

			// Démarrer le monitoring
			system.startMonitoring();

			logs.log('✅ DJ-4LED initialized', 'success', 'system');
			return { success: true, message: 'Application initialized successfully' };
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			logs.log(`❌ Initialization failed: ${errorMessage}`, 'error', 'system');
			return { success: false, message: `Initialization failed: ${errorMessage}` };
		}
	};

	// Arrêt simple
	const shutdownApp = async (): Promise<ActionResult> => {
		try {
			logs.log('🛑 Shutting down DJ-4LED...', 'info', 'system');

			await system.shutdown();
			audio.cleanup();
			frames.cleanup();

			logs.log('✅ DJ-4LED shut down', 'success', 'system');
			return { success: true, message: 'Application shut down successfully' };
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			logs.log(`❌ Shutdown failed: ${errorMessage}`, 'error', 'system');
			return { success: false, message: `Shutdown failed: ${errorMessage}` };
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

		// Actions globales
		initializeApp,
		shutdownApp,
	};
};
