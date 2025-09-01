import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { onMounted, onUnmounted } from 'vue';

import { useColorsStore } from '@/stores/colors';

import type { ActionResult, CustomColor } from '../types';

export function useColors() {
	const store = useColorsStore();

	let unlistenColorMode: UnlistenFn | null = null;
	let unlistenCustomColor: UnlistenFn | null = null;

	// ===== API CALLS =====
	const setColorMode = async (mode: string): Promise<ActionResult> => {
		if (!store.validateMode(mode)) {
			return {
				success: false,
				message: `Invalid mode: ${mode}. Valid modes: rainbow, fire, ocean, sunset, custom`,
			};
		}

		store.setLoading(true);
		try {
			const result = await invoke<any>('effects_set_color_mode', { mode });
			store.setCurrentMode(mode);
			console.log(`🌈 Color mode changed to: ${mode}`);
			return {
				success: true,
				message: result.message || `Color mode set to ${mode}`,
				data: result,
			};
		} catch (err) {
			console.error(`❌ Failed to set color mode: ${err}`);
			return { success: false, message: String(err) };
		} finally {
			store.setLoading(false);
		}
	};

	const getColorMode = async (): Promise<ActionResult> => {
		try {
			const config = await invoke<any>('effects_get_color_mode');
			store.updateColorConfig(config);
			console.log(`🔍 Color config retrieved:`, config);
			return {
				success: true,
				message: 'Color configuration retrieved successfully',
				data: config,
			};
		} catch (err) {
			console.error(`❌ Failed to get color mode: ${err}`);
			return { success: false, message: String(err) };
		}
	};

	const setCustomColor = async (color: CustomColor): Promise<ActionResult> => {
		const validColor = store.validateColor(color);

		store.setLoading(true);
		try {
			const { r, g, b } = validColor;
			const result = await invoke<any>('effects_set_custom_color', { r, g, b });
			store.setCustomColor(validColor);
			console.log(
				`🎨 Custom color set to RGB(${Math.round(r * 255)}, ${Math.round(g * 255)}, ${Math.round(b * 255)})`
			);
			return {
				success: true,
				message: result.message || `Custom color updated`,
				data: result,
			};
		} catch (err) {
			console.error(`❌ Failed to set custom color: ${err}`);
			return { success: false, message: String(err) };
		} finally {
			store.setLoading(false);
		}
	};

	const getCustomColor = async (): Promise<ActionResult> => {
		try {
			const color = await invoke<any>('effects_get_custom_color');
			if (color.r !== undefined && color.g !== undefined && color.b !== undefined) {
				store.setCustomColor({ r: color.r, g: color.g, b: color.b });
				console.log(`🎨 Custom color retrieved:`, color);
			}
			return {
				success: true,
				message: 'Custom color retrieved successfully',
				data: color,
			};
		} catch (err) {
			console.error(`❌ Failed to get custom color: ${err}`);
			return { success: false, message: String(err) };
		}
	};

	const setColorFromHex = async (hex: string): Promise<ActionResult> => {
		const cleanHex = hex.replace('#', '');
		if (!/^[0-9A-Fa-f]{6}$/.test(cleanHex)) {
			return {
				success: false,
				message: 'Invalid hex format. Use format: #RRGGBB (e.g., #FF8000)',
			};
		}

		const r = parseInt(cleanHex.substring(0, 2), 16) / 255;
		const g = parseInt(cleanHex.substring(2, 4), 16) / 255;
		const b = parseInt(cleanHex.substring(4, 6), 16) / 255;

		console.log(
			`🎨 Converting hex ${hex} to RGB(${Math.round(r * 255)}, ${Math.round(g * 255)}, ${Math.round(b * 255)})`
		);
		return await setCustomColor({ r, g, b });
	};

	// ===== EVENT HANDLERS =====
	const handleColorModeChanged = (event: any) => {
		const data = event.payload;
		console.log(`🌈 [EVENT] Color mode changed:`, data);
		if (data.mode && store.validateMode(data.mode)) {
			store.setCurrentMode(data.mode);
		}
	};

	const handleCustomColorChanged = (event: any) => {
		const data = event.payload;
		console.log(`🎨 [EVENT] Custom color changed:`, data);
		if (data.r !== undefined && data.g !== undefined && data.b !== undefined) {
			store.setCustomColor({ r: data.r, g: data.g, b: data.b });
		}
	};

	// ===== LIFECYCLE =====
	const setupListeners = async (): Promise<void> => {
		try {
			console.log(`🔧 Setting up color event listeners...`);
			unlistenColorMode = await listen('color_mode_changed', handleColorModeChanged);
			unlistenCustomColor = await listen('custom_color_changed', handleCustomColorChanged);
			console.log(`✅ Color event listeners setup complete`);
		} catch (err) {
			console.error('❌ Failed to setup color listeners:', err);
		}
	};

	const cleanup = (): void => {
		console.log(`🧹 Cleaning up color listeners...`);
		unlistenColorMode?.();
		unlistenCustomColor?.();
		unlistenColorMode = null;
		unlistenCustomColor = null;
	};

	const initialize = async (): Promise<void> => {
		console.log('🌈 Initializing colors composable...');
		try {
			await setupListeners();

			// Get initial state from backend
			const [modeResult, colorResult] = await Promise.allSettled([getColorMode(), getCustomColor()]);

			if (modeResult.status === 'fulfilled' && modeResult.value.success) {
				console.log('✅ Color mode initialized');
			}
			if (colorResult.status === 'fulfilled' && colorResult.value.success) {
				console.log('✅ Custom color initialized');
			}

			console.log('✅ Colors composable initialized');
		} catch (err) {
			console.error('❌ Failed to initialize colors:', err);
		}
	};

	// ===== UTILITIES =====
	const syncWithBackend = async (): Promise<ActionResult> => {
		console.log('🔄 Syncing colors with backend...');
		try {
			await Promise.all([getColorMode(), getCustomColor()]);
			return {
				success: true,
				message: 'Colors synchronized with backend successfully',
			};
		} catch (err) {
			return {
				success: false,
				message: `Failed to sync: ${String(err)}`,
			};
		}
	};

	onMounted(() => {
		console.log('🌈 Colors composable mounted');
		initialize();
	});

	onUnmounted(() => {
		console.log('💀 Colors composable unmounting');
		cleanup();
	});

	return {
		// Store
		...store,

		// Methods
		setColorMode,
		getColorMode,
		setCustomColor,
		getCustomColor,
		setColorFromHex,
		initialize,
		syncWithBackend,
	};
}
