import { invoke } from '@tauri-apps/api/core';
import { useColorsStore } from '../stores/colors';
import { useLogsStore } from '../stores/logs';
import type { ActionResult, ColorConfig, CustomColor } from '../types';

export function useColors() {
	const colorsStore = useColorsStore();
	const logsStore = useLogsStore();

	// Définir le mode couleur
	const setColorMode = async (mode: string): Promise<ActionResult> => {
		colorsStore.setLoading(true);
		try {
			const result = await invoke<string>('set_color_mode', { mode });
			colorsStore.setCurrentMode(mode);
			logsStore.addLog(`🌈 Color mode changed to: ${mode}`, 'success', 'effects');
			return { success: true, message: result };
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			logsStore.addLog(`Failed to set color mode "${mode}": ${errorMessage}`, 'error', 'effects');
			return { success: false, message: `❌ Mode error: ${errorMessage}` };
		} finally {
			colorsStore.setLoading(false);
		}
	};

	// Définir une couleur personnalisée
	const setCustomColor = async (color?: CustomColor): Promise<ActionResult> => {
		const targetColor = color || colorsStore.customColor;
		colorsStore.setLoading(true);
		try {
			const { r, g, b } = targetColor;
			const result = await invoke<string>('set_custom_color', { r, g, b });
			colorsStore.setCustomColor(targetColor);
			logsStore.addLog(
				`🎨 Custom color set to RGB(${Math.round(r * 255)}, ${Math.round(g * 255)}, ${Math.round(b * 255)})`,
				'success',
				'effects'
			);
			return { success: true, message: result };
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			logsStore.addLog(`Failed to set custom color: ${errorMessage}`, 'error', 'effects');
			return { success: false, message: `❌ Color error: ${errorMessage}` };
		} finally {
			colorsStore.setLoading(false);
		}
	};

	// Récupérer la configuration couleur actuelle
	const getColorMode = async (): Promise<ActionResult> => {
		try {
			const config = await invoke<ColorConfig>('get_color_mode');
			colorsStore.updateColorConfig(config);
			return { success: true, message: 'Color configuration retrieved', data: config };
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			logsStore.addLog(`Failed to get color mode: ${errorMessage}`, 'warning', 'effects');
			return { success: false, message: `Failed to get color mode: ${errorMessage}` };
		}
	};

	// Récupérer la couleur personnalisée
	const getCustomColor = async (): Promise<ActionResult> => {
		try {
			const color = await invoke<any>('get_custom_color');
			colorsStore.setCustomColor({ r: color.r, g: color.g, b: color.b });
			return { success: true, message: 'Custom color retrieved', data: color };
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			logsStore.addLog(`Failed to get custom color: ${errorMessage}`, 'warning', 'effects');
			return { success: false, message: `Failed to get custom color: ${errorMessage}` };
		}
	};

	return {
		// Store state access
		currentMode: colorsStore.currentMode,
		customColor: colorsStore.customColor,
		availableModes: colorsStore.availableModes,
		loading: colorsStore.loading,
		colorPreviewStyle: colorsStore.colorPreviewStyle,
		hexColor: colorsStore.hexColor,
		currentModeInfo: colorsStore.currentModeInfo,
		isCustomMode: colorsStore.isCustomMode,

		// Actions
		setColorMode,
		setCustomColor,
		getColorMode,
		getCustomColor,
		reset: colorsStore.reset,
	};
}
