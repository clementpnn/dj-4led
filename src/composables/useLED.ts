import { invoke } from '@tauri-apps/api/core';
import { TEST_PATTERNS } from '../config';
import { useLEDStore } from '../stores/led';
import { useLogsStore } from '../stores/logs';
import type { ActionResult, LEDStats } from '../types';

export function useLED() {
	const ledStore = useLEDStore();
	const logsStore = useLogsStore();

	// Démarrer la sortie LED
	const startLEDOutput = async (mode: 'simulator' | 'production' = 'simulator'): Promise<ActionResult> => {
		ledStore.setLoading(true);
		try {
			const result = await invoke<string>('start_led_output', { mode });
			ledStore.setRunning(true);
			ledStore.setMode(mode);
			await getLEDStats(); // Refresh stats
			logsStore.addLog(`💡 LED output started in ${mode} mode`, 'success', 'led');
			return { success: true, message: result };
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			logsStore.addLog(`Failed to start LED output: ${errorMessage}`, 'error', 'led');
			return { success: false, message: `❌ LED start error: ${errorMessage}` };
		} finally {
			ledStore.setLoading(false);
		}
	};

	// Arrêter la sortie LED
	const stopLEDOutput = async (): Promise<ActionResult> => {
		ledStore.setLoading(true);
		try {
			const result = await invoke<string>('stop_led_output');
			ledStore.setRunning(false);
			await getLEDStats(); // Refresh stats
			logsStore.addLog('🛑 LED output stopped', 'info', 'led');
			return { success: true, message: result };
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			logsStore.addLog(`Failed to stop LED output: ${errorMessage}`, 'error', 'led');
			return { success: false, message: `❌ LED stop error: ${errorMessage}` };
		} finally {
			ledStore.setLoading(false);
		}
	};

	// Vérifier si les LEDs sont en cours d'exécution
	const checkLEDRunning = async (): Promise<boolean> => {
		try {
			const running = await invoke<boolean>('is_led_running');
			ledStore.setRunning(running);
			return running;
		} catch (error) {
			logsStore.addLog('Failed to check LED status', 'warning', 'led');
			return false;
		}
	};

	// Définir la luminosité
	const setLEDBrightness = async (brightnessValue: number): Promise<ActionResult> => {
		try {
			const result = await invoke<string>('set_led_brightness', { brightness: brightnessValue });
			ledStore.setBrightness(brightnessValue);
			logsStore.addLog(`🔆 LED brightness set to ${Math.round(brightnessValue * 100)}%`, 'info', 'led');
			return { success: true, message: result };
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			logsStore.addLog(`Failed to set LED brightness: ${errorMessage}`, 'error', 'led');
			return { success: false, message: `❌ Brightness error: ${errorMessage}` };
		}
	};

	// Récupérer la luminosité actuelle
	const getLEDBrightness = async (): Promise<number> => {
		try {
			const brightnessValue = await invoke<number>('get_led_brightness');
			ledStore.setBrightness(brightnessValue);
			return brightnessValue;
		} catch (error) {
			logsStore.addLog('Failed to get LED brightness', 'warning', 'led');
			return ledStore.brightness;
		}
	};

	// Tester un pattern LED
	const testLEDPattern = async (pattern: string): Promise<ActionResult> => {
		const validPattern = TEST_PATTERNS.find((p) => p.value === pattern);
		if (!validPattern) {
			const validPatterns = TEST_PATTERNS.map((p) => p.value).join(', ');
			return {
				success: false,
				message: `Invalid pattern. Available: ${validPatterns}`,
			};
		}

		try {
			const result = await invoke<string>('test_led_pattern', { pattern });
			logsStore.addLog(`🎨 LED pattern test: ${validPattern.label}`, 'info', 'led');
			return { success: true, message: result };
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			logsStore.addLog(`Failed to test LED pattern "${pattern}": ${errorMessage}`, 'error', 'led');
			return { success: false, message: `❌ Pattern error: ${errorMessage}` };
		}
	};

	// Récupérer les contrôleurs LED
	const getLEDControllers = async (): Promise<ActionResult> => {
		try {
			const controllerList = await invoke<string[]>('get_led_controllers');
			ledStore.setControllers(
				controllerList.map((address, index) => ({
					id: `controller-${index}`,
					address,
					port: 6454,
					status: 'connected' as const,
					lastSeen: Date.now(),
				}))
			);
			return { success: true, message: `Found ${controllerList.length} controllers`, data: controllerList };
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			logsStore.addLog(`Failed to get LED controllers: ${errorMessage}`, 'warning', 'led');
			return { success: false, message: `Failed to get LED controllers: ${errorMessage}` };
		}
	};

	// Récupérer les statistiques LED
	const getLEDStats = async (): Promise<ActionResult> => {
		try {
			const ledStats = await invoke<LEDStats>('get_led_stats');
			ledStore.setStats(ledStats);
			return { success: true, message: 'LED stats retrieved', data: ledStats };
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			logsStore.addLog(`Failed to get LED stats: ${errorMessage}`, 'warning', 'led');
			return { success: false, message: `Failed to get LED stats: ${errorMessage}` };
		}
	};

	return {
		// Store state access
		stats: ledStore.stats,
		controllers: ledStore.controllers,
		brightness: ledStore.brightness,
		loading: ledStore.loading,
		isRunning: ledStore.isRunning,
		currentMode: ledStore.currentMode,
		frameSize: ledStore.frameSize,
		matrixSize: ledStore.matrixSize,
		controllerCount: ledStore.controllerCount,
		connectedControllers: ledStore.connectedControllers,
		isHealthy: ledStore.isHealthy,

		// Actions
		startLEDOutput,
		stopLEDOutput,
		checkLEDRunning,
		setLEDBrightness,
		getLEDBrightness,
		testLEDPattern,
		getLEDControllers,
		getLEDStats,
		reset: ledStore.reset,
	};
}
