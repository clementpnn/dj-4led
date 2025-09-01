// composables/usePresets.ts
import { onMounted } from 'vue';

import { useColorsStore } from '@/stores/colors';
import { useEffectsStore } from '@/stores/effects';
import { usePresetsStore } from '@/stores/presets';
import type { ActionResult, Preset, PresetConfig } from '@/types';

interface ComposableSet {
	audio?: any;
	effects?: any;
	colors?: any;
	led?: any;
}

export function usePresets() {
	// Store instances
	const store = usePresetsStore();
	const colorsStore = useColorsStore();
	const effectsStore = useEffectsStore();

	// ===== PRESET MANAGEMENT =====
	const createPreset = async (
		name: string,
		description: string,
		composables: ComposableSet
	): Promise<ActionResult> => {
		store.setLoading(true);

		try {
			// Validate name
			const trimmedName = name.trim();
			if (!trimmedName || trimmedName.length < 2) {
				return { success: false, message: 'Name must be at least 2 characters' };
			}

			// Check if name exists using centralized store method
			if (store.nameExists(trimmedName)) {
				return { success: false, message: 'Preset name already exists' };
			}

			// Capture current configuration
			const config = captureCurrentConfig(composables);

			// Create preset object
			const preset: Preset = {
				id: `preset-${Date.now()}-${Math.random().toString(36).substr(2, 5)}`,
				name: trimmedName,
				description: description.trim() || `Custom preset created on ${new Date().toLocaleDateString()}`,
				config,
				createdAt: Date.now(),
				tags: ['custom'],
			};

			const success = store.addPreset(preset);

			if (success) {
				return { success: true, message: 'Preset created successfully', data: preset };
			} else {
				return { success: false, message: 'Failed to save preset' };
			}
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : 'Failed to create preset';
			return { success: false, message: errorMessage };
		} finally {
			store.setLoading(false);
		}
	};

	const applyPreset = async (presetId: string, composables: ComposableSet): Promise<ActionResult> => {
		store.setApplying(true);

		try {
			const preset = store.getPresetById(presetId);
			if (!preset) {
				return { success: false, message: 'Preset not found' };
			}

			// Apply settings
			await Promise.all([
				applyAudioSettings(composables.audio, preset.config.audio),
				applyEffectSettings(composables.effects, preset.config.effect),
				applyColorSettings(composables.colors, preset.config.color),
				applyLedSettings(composables.led, preset.config.led),
			]);

			store.setCurrentPreset(preset);
			return { success: true, message: `Preset "${preset.name}" applied successfully` };
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : 'Failed to apply preset';
			return { success: false, message: errorMessage };
		} finally {
			store.setApplying(false);
		}
	};

	const duplicatePreset = (presetId: string, newName: string): ActionResult => {
		const result = store.duplicatePreset(presetId, newName);

		if (result) {
			return { success: true, message: 'Preset duplicated successfully', data: result };
		} else {
			return { success: false, message: 'Failed to duplicate preset' };
		}
	};

	const deletePreset = (presetId: string): ActionResult => {
		const result = store.deletePreset(presetId);

		if (result) {
			return { success: true, message: 'Preset deleted successfully' };
		} else {
			return { success: false, message: 'Failed to delete preset' };
		}
	};

	// ===== HELPER FUNCTIONS =====
	const applyAudioSettings = async (audio: any, config: any) => {
		if (audio?.setGain && config?.gain !== undefined) {
			await audio.setGain(config.gain);
		}
	};

	const applyColorSettings = async (colors: any, config: any) => {
		console.log('🌈 [PRESETS] Applying color settings:', config);

		if (config?.mode) {
			// Use store directly for more reliable updates
			colorsStore.setCurrentMode(config.mode);

			if (config.mode === 'custom' && config.customColor) {
				colorsStore.setCustomColor(config.customColor);
			}

			// Also try composable method as fallback (but avoid loops)
			if (colors?.setColorMode && !colors._applyingPreset) {
				colors._applyingPreset = true;
				try {
					await colors.setColorMode(config.mode);
					if (config.mode === 'custom' && config.customColor && colors.setCustomColor) {
						await colors.setCustomColor(config.customColor);
					}
				} finally {
					colors._applyingPreset = false;
				}
			}
		}
	};

	const applyEffectSettings = async (effects: any, config: any) => {
		console.log('🎇 [PRESETS] Applying effect settings:', config);

		if (config?.id !== undefined) {
			// Use store directly for more reliable updates
			effectsStore.setCurrentEffect({
				id: config.id,
				name: config.name || 'SpectrumBars',
				transitioning: false,
				transition_progress: 1,
			});

			// Also try composable method as fallback (but avoid loops)
			if (effects?.setEffect && !effects._applyingPreset) {
				effects._applyingPreset = true;
				try {
					await effects.setEffect(config.id);
				} finally {
					effects._applyingPreset = false;
				}
			}
		}
	};

	const applyLedSettings = async (led: any, config: any) => {
		if (led?.setBrightness && config?.brightness !== undefined) {
			await led.setBrightness(config.brightness);
		}
	};

	const captureCurrentConfig = (composables: ComposableSet): PresetConfig => {
		console.log('📸 [PRESETS] Capturing current configuration...');

		const config: PresetConfig = {
			effect: { id: 0, name: 'SpectrumBars' },
			color: { mode: 'rainbow' },
			audio: { gain: 1.0 },
			led: { brightness: 0.8, mode: 'simulator' },
		};

		try {
			// Capture effect from effects store
			if (effectsStore.currentEffect) {
				config.effect = {
					id: effectsStore.currentEffect.id,
					name: effectsStore.currentEffect.name,
				};
				console.log('🎇 [PRESETS] Captured effect:', config.effect);
			}

			// Capture color from colors store
			config.color = {
				mode: colorsStore.currentMode,
			};

			// If custom mode, capture the custom color
			if (colorsStore.currentMode === 'custom') {
				config.color.customColor = {
					r: colorsStore.customColor.r,
					g: colorsStore.customColor.g,
					b: colorsStore.customColor.b,
				};
			}
			console.log('🌈 [PRESETS] Captured color:', config.color);

			// Capture audio from composable (fallback)
			if (composables.audio?.state?.currentGain !== undefined) {
				config.audio.gain = composables.audio.state.currentGain;
			} else if (composables.audio?.currentGain !== undefined) {
				config.audio.gain = composables.audio.currentGain;
			}
			console.log('🔊 [PRESETS] Captured audio:', config.audio);

			// Capture LED from composable (fallback)
			if (composables.led?.brightness !== undefined) {
				config.led.brightness = composables.led.brightness;
			}
			if (composables.led?.currentMode) {
				config.led.mode = composables.led.currentMode;
			}
			console.log('💡 [PRESETS] Captured LED:', config.led);
		} catch (err) {
			console.warn('⚠️ [PRESETS] Some settings could not be captured, using defaults:', err);
		}

		console.log('✅ [PRESETS] Final captured config:', config);
		return config;
	};

	// ===== EXPORT/IMPORT =====
	const exportPresets = (): ActionResult => {
		const result = store.exportPresets(false);

		if (!result.success || !result.data) {
			return { success: false, message: result.message };
		}

		try {
			const blob = new Blob([result.data], { type: 'application/json' });
			const url = URL.createObjectURL(blob);
			const a = document.createElement('a');
			a.href = url;
			a.download = `dj4led-presets-${new Date().toISOString().split('T')[0]}.json`;
			document.body.appendChild(a);
			a.click();
			document.body.removeChild(a);
			URL.revokeObjectURL(url);

			return { success: true, message: result.message };
		} catch (err) {
			return { success: false, message: 'Failed to download export file' };
		}
	};

	const importPresets = (file: File): Promise<ActionResult> => {
		return new Promise((resolve) => {
			const reader = new FileReader();

			reader.onload = (e) => {
				try {
					const data = e.target?.result as string;
					const result = store.importPresets(data);

					if (result.success) {
						resolve({
							success: true,
							message: result.message,
							data: { imported: result.imported, skipped: result.skipped },
						});
					} else {
						resolve({ success: false, message: result.message });
					}
				} catch (err) {
					resolve({ success: false, message: 'Failed to import presets' });
				}
			};

			reader.onerror = () => resolve({ success: false, message: 'Failed to read file' });
			reader.readAsText(file);
		});
	};

	// ===== LIFECYCLE =====
	onMounted(() => {
		store.loadFromStorage();
	});

	// ===== PUBLIC API - Return only methods, not store data =====
	return {
		// Actions only - no store data
		createPreset,
		applyPreset,
		deletePreset,
		duplicatePreset,

		// Utils
		exportPresets,
		importPresets,
	};
}
