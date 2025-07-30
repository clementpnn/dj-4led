// composables/usePresets.ts
import { onMounted, readonly, ref } from 'vue';
import { usePresetsStore } from '../stores/presets';
import type { ActionResult, Preset, PresetConfig } from '../types';

interface ComposableSet {
	audio?: any;
	effects?: any;
	colors?: any;
	led?: any;
}

export function usePresets() {
	// Store instance
	const presetsStore = usePresetsStore();

	// Local state
	const isApplying = ref(false);

	// ===== PRESET MANAGEMENT =====
	const createPreset = async (
		name: string,
		description: string,
		composables: ComposableSet
	): Promise<ActionResult> => {
		try {
			if (!name || name.length < 2) {
				return { success: false, message: 'Name must be at least 2 characters' };
			}

			const config = captureCurrentConfig(composables);

			const preset: Preset = {
				id: `preset-${Date.now()}-${Math.random().toString(36).substr(2, 5)}`,
				name: name.trim(),
				description: description.trim() || `Custom preset created on ${new Date().toLocaleDateString()}`,
				config,
				createdAt: Date.now(),
				tags: ['custom'],
			};

			const success = presetsStore.addPreset(preset);
			if (success) {
				return { success: true, message: 'Preset created successfully', data: preset };
			} else {
				return { success: false, message: 'Preset name already exists' };
			}
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : 'Failed to create preset';
			return { success: false, message: errorMessage };
		}
	};

	const applyPreset = async (presetId: string, composables: ComposableSet): Promise<ActionResult> => {
		isApplying.value = true;

		try {
			const preset = presetsStore.getPresetById(presetId);
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

			presetsStore.setCurrentPreset(preset);
			return { success: true, message: `Preset "${preset.name}" applied successfully` };
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : 'Failed to apply preset';
			return { success: false, message: errorMessage };
		} finally {
			isApplying.value = false;
		}
	};

	// ===== HELPER FUNCTIONS =====
	const applyAudioSettings = async (audio: any, config: any) => {
		if (audio?.setGain && config?.gain !== undefined) {
			await audio.setGain(config.gain);
		}
	};

	const applyEffectSettings = async (effects: any, config: any) => {
		if (effects?.setEffect && config?.id !== undefined) {
			await effects.setEffect(config.id);
		}
	};

	const applyColorSettings = async (colors: any, config: any) => {
		if (colors?.setColorMode && config?.mode) {
			await colors.setColorMode(config.mode);

			if (config.mode === 'custom' && config.customColor && colors.setCustomColor) {
				await colors.setCustomColor(config.customColor);
			}
		}
	};

	const applyLedSettings = async (led: any, config: any) => {
		if (led?.setBrightness && config?.brightness !== undefined) {
			await led.setBrightness(config.brightness);
		}
	};

	const captureCurrentConfig = (composables: ComposableSet): PresetConfig => {
		const config: PresetConfig = {
			effect: { id: 0, name: 'SpectrumBars' },
			color: { mode: 'rainbow' },
			audio: { gain: 1.0 },
			led: { brightness: 0.8, mode: 'simulator' },
		};

		try {
			// Capture audio
			if (composables.audio?.state?.currentGain !== undefined) {
				config.audio.gain = composables.audio.state.currentGain;
			}

			// Capture effect
			if (composables.effects?.currentEffect) {
				config.effect = {
					id: composables.effects.currentEffect.id || 0,
					name: composables.effects.currentEffect.name || 'SpectrumBars',
				};
			}

			// Capture color
			if (composables.colors?.currentMode) {
				config.color.mode = composables.colors.currentMode;
				if (composables.colors.currentMode === 'custom' && composables.colors.customColor) {
					config.color.customColor = composables.colors.customColor;
				}
			}

			// Capture LED
			if (composables.led?.brightness !== undefined) {
				config.led.brightness = composables.led.brightness;
			}
			if (composables.led?.currentMode) {
				config.led.mode = composables.led.currentMode;
			}
		} catch (err) {
			console.warn('Some settings could not be captured, using defaults');
		}

		return config;
	};

	// ===== EXPORT/IMPORT =====
	const exportPresets = (): ActionResult => {
		try {
			const data = presetsStore.exportPresets(false);

			const blob = new Blob([data], { type: 'application/json' });
			const url = URL.createObjectURL(blob);
			const a = document.createElement('a');
			a.href = url;
			a.download = `dj4led-presets-${new Date().toISOString().split('T')[0]}.json`;
			document.body.appendChild(a);
			a.click();
			document.body.removeChild(a);
			URL.revokeObjectURL(url);

			return { success: true, message: `${presetsStore.presets.length} presets exported` };
		} catch (err) {
			return { success: false, message: 'Failed to export presets' };
		}
	};

	const importPresets = (file: File): Promise<ActionResult> => {
		return new Promise((resolve) => {
			const reader = new FileReader();

			reader.onload = (e) => {
				try {
					const data = e.target?.result as string;
					const result = presetsStore.importPresets(data);

					resolve({
						success: true,
						message: `${result.imported} presets imported${result.skipped > 0 ? `, ${result.skipped} skipped` : ''}`,
						data: result,
					});
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
		presetsStore.loadFromStorage();
	});

	// ===== PUBLIC API =====
	return {
		// Store access
		presets: presetsStore.presets,
		allPresets: presetsStore.allPresets,
		currentPreset: presetsStore.currentPreset,
		loading: presetsStore.loading,
		availableTags: presetsStore.availableTags,

		// Local state
		isApplying: readonly(isApplying),

		// Actions
		createPreset,
		applyPreset,
		deletePreset: presetsStore.deletePreset,
		duplicatePreset: presetsStore.duplicatePreset,

		// Utils
		exportPresets,
		importPresets,
		getPresetById: presetsStore.getPresetById,
		clearLogs: presetsStore.clearLogs,
	};
}
