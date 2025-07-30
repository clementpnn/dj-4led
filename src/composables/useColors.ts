// composables/useColors.ts
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { onMounted, onUnmounted } from 'vue';
import { useColorsStore } from '../stores/colors';
import type { ActionResult, CustomColor } from '../types';

export function useColors() {
	// Store instance
	const colorsStore = useColorsStore();

	// Event listeners references
	let unlistenColorModeChanged: UnlistenFn | null = null;
	let unlistenCustomColorChanged: UnlistenFn | null = null;

	// ===== COLOR MODE ACTIONS =====

	const setColorMode = async (mode: string): Promise<ActionResult> => {
		if (!colorsStore.availableModes.some((m) => m.value === mode)) {
			return {
				success: false,
				message: `Invalid color mode "${mode}". Available modes: ${colorsStore.availableModes.map((m) => m.value).join(', ')}`,
			};
		}

		colorsStore.setLoading(true);

		try {
			const result = await invoke<any>('effects_set_color_mode', { mode });
			colorsStore.setCurrentMode(mode);
			console.log(`🌈 Color mode changed to "${mode}":`, result);

			return {
				success: true,
				message: result.message || `Color mode set to ${mode}`,
				data: result,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			console.error(`❌ Failed to set color mode "${mode}":`, errorMessage);
			return { success: false, message: errorMessage };
		} finally {
			colorsStore.setLoading(false);
		}
	};

	const getColorMode = async (): Promise<ActionResult> => {
		try {
			const config = await invoke<any>('effects_get_color_mode');
			colorsStore.updateColorConfig(config);

			console.log('🌈 Color configuration retrieved:', config);
			return {
				success: true,
				message: 'Color configuration retrieved',
				data: config,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			console.warn('Failed to get color mode:', errorMessage);
			return { success: false, message: errorMessage };
		}
	};

	// ===== CUSTOM COLOR ACTIONS =====

	const setCustomColor = async (color?: CustomColor): Promise<ActionResult> => {
		const targetColor = color || colorsStore.customColor;

		// Validate color values
		if (
			targetColor.r < 0 ||
			targetColor.r > 1 ||
			targetColor.g < 0 ||
			targetColor.g > 1 ||
			targetColor.b < 0 ||
			targetColor.b > 1
		) {
			return {
				success: false,
				message: 'Color values must be between 0.0 and 1.0',
			};
		}

		colorsStore.setLoading(true);

		try {
			const { r, g, b } = targetColor;
			const result = await invoke<any>('effects_set_custom_color', { r, g, b });
			colorsStore.setCustomColor(targetColor);

			console.log(
				`🎨 Custom color set to RGB(${Math.round(r * 255)}, ${Math.round(g * 255)}, ${Math.round(b * 255)}):`,
				result
			);

			return {
				success: true,
				message:
					result.message ||
					`Custom color set to RGB(${Math.round(r * 255)}, ${Math.round(g * 255)}, ${Math.round(b * 255)})`,
				data: result,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			console.error('❌ Failed to set custom color:', errorMessage);
			return { success: false, message: errorMessage };
		} finally {
			colorsStore.setLoading(false);
		}
	};

	const getCustomColor = async (): Promise<ActionResult> => {
		try {
			const color = await invoke<any>('effects_get_custom_color');

			if (color.r !== undefined && color.g !== undefined && color.b !== undefined) {
				colorsStore.setCustomColor({
					r: color.r,
					g: color.g,
					b: color.b,
				});
			}

			console.log('🎨 Custom color retrieved:', color);
			return {
				success: true,
				message: 'Custom color retrieved',
				data: color,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			console.warn('Failed to get custom color:', errorMessage);
			return { success: false, message: errorMessage };
		}
	};

	// ===== UTILITY FUNCTIONS =====

	const setColorFromHex = async (hex: string): Promise<ActionResult> => {
		// Remove # if present
		const cleanHex = hex.replace('#', '');

		if (!/^[0-9A-Fa-f]{6}$/.test(cleanHex)) {
			return { success: false, message: 'Invalid hex color format. Use #RRGGBB' };
		}

		const r = parseInt(cleanHex.substr(0, 2), 16) / 255;
		const g = parseInt(cleanHex.substr(2, 2), 16) / 255;
		const b = parseInt(cleanHex.substr(4, 2), 16) / 255;

		return await setCustomColor({ r, g, b });
	};

	const setColorFromRGB255 = async (r: number, g: number, b: number): Promise<ActionResult> => {
		if (r < 0 || r > 255 || g < 0 || g > 255 || b < 0 || b > 255) {
			return { success: false, message: 'RGB values must be between 0 and 255' };
		}

		return await setCustomColor({
			r: r / 255,
			g: g / 255,
			b: b / 255,
		});
	};

	const getPresetColors = () => {
		return [
			{ name: 'Red', hex: '#FF0000', rgb: { r: 1.0, g: 0.0, b: 0.0 } },
			{ name: 'Green', hex: '#00FF00', rgb: { r: 0.0, g: 1.0, b: 0.0 } },
			{ name: 'Blue', hex: '#0000FF', rgb: { r: 0.0, g: 0.0, b: 1.0 } },
			{ name: 'Yellow', hex: '#FFFF00', rgb: { r: 1.0, g: 1.0, b: 0.0 } },
			{ name: 'Magenta', hex: '#FF00FF', rgb: { r: 1.0, g: 0.0, b: 1.0 } },
			{ name: 'Cyan', hex: '#00FFFF', rgb: { r: 0.0, g: 1.0, b: 1.0 } },
			{ name: 'White', hex: '#FFFFFF', rgb: { r: 1.0, g: 1.0, b: 1.0 } },
			{ name: 'Orange', hex: '#FF8000', rgb: { r: 1.0, g: 0.5, b: 0.0 } },
			{ name: 'Purple', hex: '#8000FF', rgb: { r: 0.5, g: 0.0, b: 1.0 } },
			{ name: 'Pink', hex: '#FF0080', rgb: { r: 1.0, g: 0.0, b: 0.5 } },
		];
	};

	// ===== EVENT HANDLERS =====

	const handleColorModeChanged = (event: any) => {
		const data = event.payload;
		if (data.mode) {
			colorsStore.setCurrentMode(data.mode);
			console.log(`🌈 Color mode changed to "${data.mode}"`);
		}
	};

	const handleCustomColorChanged = (event: any) => {
		const data = event.payload;
		if (data.r !== undefined && data.g !== undefined && data.b !== undefined) {
			colorsStore.setCustomColor({
				r: data.r,
				g: data.g,
				b: data.b,
			});
			console.log(
				`🎨 Custom color changed to RGB(${Math.round(data.r * 255)}, ${Math.round(data.g * 255)}, ${Math.round(data.b * 255)})`
			);
		}
	};

	// ===== EVENT LISTENERS SETUP =====

	const setupListeners = async (): Promise<void> => {
		try {
			unlistenColorModeChanged = await listen('color_mode_changed', handleColorModeChanged);
			unlistenCustomColorChanged = await listen('custom_color_changed', handleCustomColorChanged);

			console.log('✅ Color event listeners setup complete');
		} catch (err) {
			console.error('❌ Failed to setup color event listeners:', err);
		}
	};

	const cleanup = (): void => {
		const listeners = [
			{ fn: unlistenColorModeChanged, name: 'color_mode_changed' },
			{ fn: unlistenCustomColorChanged, name: 'custom_color_changed' },
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

		unlistenColorModeChanged = null;
		unlistenCustomColorChanged = null;
	};

	// ===== INITIALIZATION =====

	const initialize = async (): Promise<void> => {
		console.log('🌈 Initializing colors composable...');

		try {
			await setupListeners();
			await getColorMode();
			await getCustomColor();

			console.log('✅ Colors composable initialized successfully');
		} catch (err) {
			console.error('❌ Failed to initialize colors composable:', err);
		}
	};

	// ===== LIFECYCLE =====

	onMounted(() => {
		console.log('🌈 Colors composable mounted');
		initialize();
	});

	onUnmounted(() => {
		console.log('💀 Colors composable unmounting');
		cleanup();
	});

	// ===== PUBLIC API =====

	return {
		// Store access
		...colorsStore,

		// Actions
		setColorMode,
		getColorMode,
		setCustomColor,
		getCustomColor,
		setColorFromHex,
		setColorFromRGB255,

		// Utilities
		getPresetColors,
		initialize,
		cleanup,
	};
}
