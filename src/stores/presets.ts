import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import type { Preset, PresetCategory, PresetConfig } from '../types';

export const usePresetsStore = defineStore('presets', () => {
	// ===== STATE =====

	const presets = ref<Preset[]>([]);
	const currentPreset = ref<Preset | null>(null);
	const categories = ref<PresetCategory[]>([]);
	const loading = ref(false);

	// Default presets
	const defaultPresets: Preset[] = [
		{
			id: 'spectrum-rainbow',
			name: 'Spectrum Rainbow',
			description: 'Classic spectrum bars with rainbow colors',
			config: {
				effect: { id: 0, name: 'SpectrumBars' },
				color: { mode: 'rainbow' },
				audio: { gain: 1.0 },
				led: { brightness: 0.8, mode: 'simulator' },
			},
			createdAt: Date.now(),
			tags: ['default', 'spectrum', 'rainbow'],
		},
		{
			id: 'fire-waves',
			name: 'Fire Waves',
			description: 'Circular waves with fire colors',
			config: {
				effect: { id: 1, name: 'CircularWave' },
				color: { mode: 'fire' },
				audio: { gain: 1.5 },
				led: { brightness: 0.9, mode: 'simulator' },
			},
			createdAt: Date.now(),
			tags: ['default', 'ambient', 'fire'],
		},
		{
			id: 'ocean-particles',
			name: 'Ocean Particles',
			description: 'Particle system with ocean colors',
			config: {
				effect: { id: 2, name: 'ParticleSystem' },
				color: { mode: 'ocean' },
				audio: { gain: 1.2 },
				led: { brightness: 0.7, mode: 'simulator' },
			},
			createdAt: Date.now(),
			tags: ['default', 'particle', 'ocean'],
		},
		{
			id: 'custom-red',
			name: 'Custom Red',
			description: 'Heartbeat effect with custom red color',
			config: {
				effect: { id: 3, name: 'Heartbeat' },
				color: { mode: 'custom', customColor: { r: 1, g: 0, b: 0 } },
				audio: { gain: 0.8 },
				led: { brightness: 1.0, mode: 'simulator' },
			},
			createdAt: Date.now(),
			tags: ['default', 'rhythm', 'custom'],
		},
	];

	// ===== GETTERS =====

	const allPresets = computed(() => [...defaultPresets, ...presets.value]);

	const customPresets = computed(() =>
		presets.value.filter((preset) => !defaultPresets.find((dp) => dp.id === preset.id))
	);

	const presetsByCategory = computed(() => {
		const categorized: Record<string, Preset[]> = {
			default: defaultPresets,
			custom: customPresets.value,
		};

		categories.value.forEach((category) => {
			categorized[category.id] = category.presets;
		});

		return categorized;
	});

	const getPresetById = computed(() => (id: string) => allPresets.value.find((preset) => preset.id === id));

	const isDefaultPreset = computed(() => (id: string) => defaultPresets.some((preset) => preset.id === id));

	// ===== ACTIONS =====

	const setPresets = (newPresets: Preset[]) => {
		presets.value = newPresets;
	};

	const addPreset = (preset: Preset) => {
		// Ensure unique ID
		if (!getPresetById.value(preset.id)) {
			presets.value.push(preset);
			saveToStorage();
		}
	};

	const updatePreset = (id: string, updates: Partial<Preset>) => {
		const index = presets.value.findIndex((p) => p.id === id);
		if (index !== -1) {
			presets.value[index] = {
				...presets.value[index],
				...updates,
				updatedAt: Date.now(),
			};
			saveToStorage();
		}
	};

	const removePreset = (id: string) => {
		// Don't allow removal of default presets
		if (isDefaultPreset.value(id)) return false;

		const index = presets.value.findIndex((p) => p.id === id);
		if (index !== -1) {
			presets.value.splice(index, 1);

			// Clear current preset if it was deleted
			if (currentPreset.value?.id === id) {
				currentPreset.value = null;
			}

			saveToStorage();
			return true;
		}
		return false;
	};

	const setCurrentPreset = (preset: Preset | null) => {
		currentPreset.value = preset;
	};

	const createPreset = (name: string, description: string, config: PresetConfig): Preset => {
		const preset: Preset = {
			id: `custom-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
			name,
			description,
			config,
			createdAt: Date.now(),
			author: 'User',
			tags: ['custom'],
		};

		addPreset(preset);
		return preset;
	};

	const duplicatePreset = (id: string, newName?: string): Preset | null => {
		const original = getPresetById.value(id);
		if (!original) return null;

		const duplicate: Preset = {
			...original,
			id: `custom-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
			name: newName || `${original.name} (Copy)`,
			createdAt: Date.now(),
			tags: [...(original.tags || []), 'duplicate'],
		};

		addPreset(duplicate);
		return duplicate;
	};

	const setLoading = (isLoading: boolean) => {
		loading.value = isLoading;
	};

	const initializeFromStorage = () => {
		try {
			const stored = localStorage.getItem('dj4led-presets');
			if (stored) {
				const parsedPresets = JSON.parse(stored);
				if (Array.isArray(parsedPresets)) {
					presets.value = parsedPresets;
				}
			}
		} catch (error) {
			console.error('Failed to load presets from storage:', error);
			presets.value = [];
		}
	};

	const saveToStorage = () => {
		try {
			localStorage.setItem('dj4led-presets', JSON.stringify(presets.value));
		} catch (error) {
			console.error('Failed to save presets to storage:', error);
		}
	};

	const exportPresets = (): string => {
		return JSON.stringify(
			{
				version: '2.0.0',
				exported: Date.now(),
				presets: presets.value,
			},
			null,
			2
		);
	};

	const importPresets = (data: string): boolean => {
		try {
			const parsed = JSON.parse(data);
			if (parsed.presets && Array.isArray(parsed.presets)) {
				// Merge with existing presets, avoiding duplicates
				parsed.presets.forEach((preset: Preset) => {
					if (!getPresetById.value(preset.id)) {
						presets.value.push(preset);
					}
				});
				saveToStorage();
				return true;
			}
		} catch (error) {
			console.error('Failed to import presets:', error);
		}
		return false;
	};

	const reset = () => {
		presets.value = [];
		currentPreset.value = null;
		categories.value = [];
		loading.value = false;
		localStorage.removeItem('dj4led-presets');
	};

	return {
		// State
		presets,
		currentPreset,
		categories,
		loading,

		// Getters
		allPresets,
		customPresets,
		presetsByCategory,
		getPresetById,
		isDefaultPreset,

		// Actions
		setPresets,
		addPreset,
		updatePreset,
		removePreset,
		setCurrentPreset,
		createPreset,
		duplicatePreset,
		setLoading,
		initializeFromStorage,
		saveToStorage,
		exportPresets,
		importPresets,
		reset,
	};
});
