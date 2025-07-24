import { useLogsStore } from '../stores/logs';
import { usePresetsStore } from '../stores/presets';
import type { ActionResult, PresetConfig } from '../types';

export function usePresets() {
	const presetsStore = usePresetsStore();
	const logsStore = useLogsStore();

	// Créer un preset simple
	const createPreset = async (name: string, description: string, config: PresetConfig): Promise<ActionResult> => {
		try {
			// Validation simple
			if (!name || name.trim().length < 3) {
				return { success: false, message: 'Name must be at least 3 characters' };
			}

			// Vérifier l'unicité
			const exists = presetsStore.allPresets.some((p) => p.name.toLowerCase() === name.toLowerCase());
			if (exists) {
				return { success: false, message: 'A preset with this name already exists' };
			}

			const preset = presetsStore.createPreset(name.trim(), description.trim(), config);
			logsStore.addLog(`🎛️ Preset created: "${preset.name}"`, 'success', 'user');

			return { success: true, message: 'Preset created successfully', data: preset };
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			logsStore.addLog(`Failed to create preset: ${errorMessage}`, 'error', 'user');
			return { success: false, message: `Failed to create preset: ${errorMessage}` };
		}
	};

	// Appliquer un preset
	const applyPreset = async (
		presetId: string,
		composables: {
			audio?: any;
			effects?: any;
			colors?: any;
			led?: any;
		}
	): Promise<ActionResult> => {
		presetsStore.setLoading(true);

		try {
			const preset = presetsStore.getPresetById(presetId);
			if (!preset) {
				return { success: false, message: 'Preset not found' };
			}

			logsStore.addLog(`🎛️ Applying preset: "${preset.name}"`, 'info', 'user');

			// Appliquer chaque configuration
			const promises = [];

			if (composables.audio) {
				promises.push(composables.audio.setAudioGain(preset.config.audio.gain));
			}

			if (composables.effects) {
				promises.push(composables.effects.setEffect(preset.config.effect.id));
			}

			if (composables.colors) {
				promises.push(composables.colors.setColorMode(preset.config.color.mode));
			}

			if (composables.led) {
				promises.push(composables.led.setLEDBrightness(preset.config.led.brightness));
			}

			await Promise.all(promises);

			// Couleur personnalisée si nécessaire
			if (preset.config.color.mode === 'custom' && preset.config.color.customColor && composables.colors) {
				await composables.colors.setCustomColor(preset.config.color.customColor);
			}

			presetsStore.setCurrentPreset(preset);
			logsStore.addLog(`✅ Preset "${preset.name}" applied successfully`, 'success', 'user');

			return {
				success: true,
				message: `Preset "${preset.name}" applied successfully`,
				data: preset,
			};
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			logsStore.addLog(`Failed to apply preset: ${errorMessage}`, 'error', 'user');
			return { success: false, message: `Failed to apply preset: ${errorMessage}` };
		} finally {
			presetsStore.setLoading(false);
		}
	};

	// Supprimer un preset
	const deletePreset = (presetId: string): ActionResult => {
		try {
			const preset = presetsStore.getPresetById(presetId);
			if (!preset) {
				return { success: false, message: 'Preset not found' };
			}

			if (presetsStore.isDefaultPreset(presetId)) {
				return { success: false, message: 'Cannot delete default preset' };
			}

			const success = presetsStore.removePreset(presetId);
			if (success) {
				logsStore.addLog(`🗑️ Preset "${preset.name}" deleted`, 'info', 'user');
				return { success: true, message: `Preset "${preset.name}" deleted successfully` };
			} else {
				return { success: false, message: 'Failed to delete preset' };
			}
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			logsStore.addLog(`Failed to delete preset: ${errorMessage}`, 'error', 'user');
			return { success: false, message: `Failed to delete preset: ${errorMessage}` };
		}
	};

	// Capturer la configuration actuelle
	const captureCurrentConfig = async (composables: {
		audio?: any;
		effects?: any;
		colors?: any;
		led?: any;
	}): Promise<PresetConfig> => {
		try {
			const [audioGain, currentEffect, colorMode, ledBrightness, customColor] = await Promise.allSettled([
				composables.audio?.getAudioGain?.() || Promise.resolve(1.0),
				composables.effects?.getCurrentEffect?.() || Promise.resolve({ data: { id: 0, name: 'SpectrumBars' } }),
				composables.colors?.getColorMode?.() || Promise.resolve({ data: { mode: 'rainbow' } }),
				composables.led?.getLEDBrightness?.() || Promise.resolve(0.8),
				composables.colors?.getCustomColor?.() || Promise.resolve({ data: { r: 1, g: 0.5, b: 0 } }),
			]);

			return {
				effect: {
					id: currentEffect.status === 'fulfilled' ? currentEffect.value?.data?.id || 0 : 0,
					name:
						currentEffect.status === 'fulfilled'
							? currentEffect.value?.data?.name || 'SpectrumBars'
							: 'SpectrumBars',
				},
				color: {
					mode: colorMode.status === 'fulfilled' ? colorMode.value?.data?.mode || 'rainbow' : 'rainbow',
					customColor: customColor.status === 'fulfilled' ? customColor.value?.data : undefined,
				},
				audio: {
					gain: audioGain.status === 'fulfilled' ? audioGain.value : 1.0,
				},
				led: {
					brightness: ledBrightness.status === 'fulfilled' ? ledBrightness.value : 0.8,
					mode: 'simulator',
				},
			};
		} catch (error) {
			logsStore.addLog('Using default configuration', 'warning', 'user');
			return {
				effect: { id: 0, name: 'SpectrumBars' },
				color: { mode: 'rainbow' },
				audio: { gain: 1.0 },
				led: { brightness: 0.8, mode: 'simulator' },
			};
		}
	};

	// Export simple
	const exportPresets = (): ActionResult => {
		try {
			const data = presetsStore.exportPresets();
			const blob = new Blob([data], { type: 'application/json' });
			const url = URL.createObjectURL(blob);
			const a = document.createElement('a');
			a.href = url;
			a.download = `dj4led-presets-${new Date().toISOString().split('T')[0]}.json`;
			document.body.appendChild(a);
			a.click();
			document.body.removeChild(a);
			URL.revokeObjectURL(url);

			logsStore.addLog(`📤 Exported ${presetsStore.customPresets.length} presets`, 'success', 'user');
			return { success: true, message: 'Presets exported successfully' };
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			logsStore.addLog(`Failed to export presets: ${errorMessage}`, 'error', 'user');
			return { success: false, message: `Failed to export presets: ${errorMessage}` };
		}
	};

	// Import simple
	const importPresets = (file: File): Promise<ActionResult> => {
		return new Promise((resolve) => {
			const reader = new FileReader();

			reader.onload = (e) => {
				try {
					const content = e.target?.result as string;
					const success = presetsStore.importPresets(content);

					if (success) {
						logsStore.addLog('📥 Presets imported successfully', 'success', 'user');
						resolve({ success: true, message: 'Presets imported successfully' });
					} else {
						resolve({ success: false, message: 'Invalid preset file format' });
					}
				} catch (error) {
					const errorMessage = error instanceof Error ? error.message : String(error);
					logsStore.addLog(`Failed to import presets: ${errorMessage}`, 'error', 'user');
					resolve({ success: false, message: `Failed to import presets: ${errorMessage}` });
				}
			};

			reader.onerror = () => {
				resolve({ success: false, message: 'Failed to read preset file' });
			};

			reader.readAsText(file);
		});
	};

	return {
		// Store state
		presets: presetsStore.presets,
		currentPreset: presetsStore.currentPreset,
		loading: presetsStore.loading,
		allPresets: presetsStore.allPresets,
		customPresets: presetsStore.customPresets,
		getPresetById: presetsStore.getPresetById,
		isDefaultPreset: presetsStore.isDefaultPreset,

		// Actions
		createPreset,
		applyPreset,
		deletePreset,
		captureCurrentConfig,
		exportPresets,
		importPresets,
		initializeFromStorage: presetsStore.initializeFromStorage,
		reset: presetsStore.reset,
	};
}
