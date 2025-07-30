// stores/presets.ts
import { defineStore } from 'pinia';
import { computed, readonly, ref } from 'vue';
import type { Preset } from '../types';

export const usePresetsStore = defineStore('presets', () => {
	// ===== STATE =====
	const presets = ref<Preset[]>([]);
	const currentPreset = ref<Preset | null>(null);
	const loading = ref(false);

	// Default presets
	const defaultPresets: Preset[] = [
		{
			id: 'party-mode',
			name: 'Party Mode',
			description: 'High energy setup with vibrant colors and dynamic effects',
			config: {
				effect: { id: 2, name: 'ParticleSystem' },
				color: { mode: 'rainbow' },
				audio: { gain: 2.0 },
				led: { brightness: 1.0, mode: 'simulator' },
			},
			createdAt: Date.now(),
			tags: ['party', 'high-energy'],
		},
		{
			id: 'chill-mode',
			name: 'Chill Mode',
			description: 'Relaxed setup with calm ocean colors',
			config: {
				effect: { id: 1, name: 'CircularWave' },
				color: { mode: 'ocean' },
				audio: { gain: 1.0 },
				led: { brightness: 0.6, mode: 'simulator' },
			},
			createdAt: Date.now(),
			tags: ['chill', 'relaxed'],
		},
		{
			id: 'focus-mode',
			name: 'Focus Mode',
			description: 'Minimal distraction setup for concentration',
			config: {
				effect: { id: 0, name: 'SpectrumBars' },
				color: { mode: 'custom', customColor: { r: 0.2, g: 0.4, b: 0.8 } },
				audio: { gain: 0.8 },
				led: { brightness: 0.4, mode: 'simulator' },
			},
			createdAt: Date.now(),
			tags: ['focus', 'minimal'],
		},
		{
			id: 'gaming-mode',
			name: 'Gaming Mode',
			description: 'Responsive setup optimized for gaming',
			config: {
				effect: { id: 6, name: 'Flames' },
				color: { mode: 'fire' },
				audio: { gain: 1.5 },
				led: { brightness: 0.8, mode: 'production' },
			},
			createdAt: Date.now(),
			tags: ['gaming', 'responsive'],
		},
	];

	// ===== GETTERS =====
	const allPresets = computed(() => [...defaultPresets, ...presets.value]);

	const availableTags = computed(() => {
		const tags = new Set<string>();
		allPresets.value.forEach((preset) => {
			preset.tags?.forEach((tag) => tags.add(tag));
		});
		return Array.from(tags).sort();
	});

	// ===== ACTIONS =====
	const addPreset = (preset: Preset) => {
		// Check for duplicate names
		const exists = allPresets.value.some((p) => p.name.toLowerCase() === preset.name.toLowerCase());
		if (exists) {
			console.warn('Preset name already exists');
			return false;
		}

		presets.value.push(preset);
		saveToStorage();
		return true;
	};

	const deletePreset = (id: string) => {
		// Prevent deletion of default presets
		if (defaultPresets.some((p) => p.id === id)) {
			console.warn('Cannot delete default preset');
			return false;
		}

		const index = presets.value.findIndex((p) => p.id === id);
		if (index !== -1) {
			presets.value.splice(index, 1);

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

	const duplicatePreset = (id: string, newName: string) => {
		const original = allPresets.value.find((p) => p.id === id);
		if (!original) {
			console.warn('Preset not found');
			return null;
		}

		const duplicate: Preset = {
			...original,
			id: `preset-${Date.now()}-${Math.random().toString(36).substr(2, 5)}`,
			name: newName,
			description: `Copy of ${original.description}`,
			createdAt: Date.now(),
			updatedAt: undefined,
		};

		const success = addPreset(duplicate);
		return success ? duplicate : null;
	};

	const getPresetById = (id: string): Preset | undefined => {
		return allPresets.value.find((p) => p.id === id);
	};

	const exportPresets = (includeDefaults = false): string => {
		const presetsToExport = includeDefaults ? allPresets.value : presets.value;

		return JSON.stringify(
			{
				version: '1.0',
				exported_at: new Date().toISOString(),
				total_presets: presetsToExport.length,
				presets: presetsToExport,
			},
			null,
			2
		);
	};

	const importPresets = (data: string): { imported: number; skipped: number } => {
		try {
			const importData = JSON.parse(data);

			if (!importData.presets || !Array.isArray(importData.presets)) {
				throw new Error('Invalid preset data format');
			}

			let imported = 0;
			let skipped = 0;

			importData.presets.forEach((presetData: any) => {
				// Validate preset structure
				if (!presetData.name || !presetData.config) {
					skipped++;
					return;
				}

				// Check for duplicate names
				const exists = allPresets.value.some((p) => p.name.toLowerCase() === presetData.name.toLowerCase());

				if (exists) {
					skipped++;
					return;
				}

				// Create new preset with new ID and timestamp
				const preset: Preset = {
					...presetData,
					id: `imported-${Date.now()}-${imported}`,
					createdAt: Date.now(),
					updatedAt: undefined,
				};

				presets.value.push(preset);
				imported++;
			});

			if (imported > 0) {
				saveToStorage();
			}

			return { imported, skipped };
		} catch (error) {
			throw new Error('Failed to parse preset data');
		}
	};

	// ===== STORAGE =====
	const saveToStorage = () => {
		try {
			localStorage.setItem('dj4led-presets', JSON.stringify(presets.value));
		} catch (error) {
			console.warn('Failed to save presets to storage:', error);
		}
	};

	const loadFromStorage = () => {
		try {
			const stored = localStorage.getItem('dj4led-presets');
			if (stored) {
				const data = JSON.parse(stored);
				if (Array.isArray(data)) {
					presets.value = data;
				}
			}
		} catch (error) {
			console.warn('Failed to load presets from storage:', error);
		}
	};

	const clearLogs = () => {
		presets.value.splice(0, presets.value.length);
		currentPreset.value = null;
		loading.value = false;

		try {
			localStorage.removeItem('dj4led-presets');
		} catch (error) {
			console.warn('Failed to clear presets storage:', error);
		}
	};

	return {
		// State
		presets: readonly(presets),
		currentPreset: readonly(currentPreset),
		loading: readonly(loading),

		// Getters
		allPresets,
		availableTags,

		// Actions
		addPreset,
		deletePreset,
		setCurrentPreset,
		duplicatePreset,
		getPresetById,
		exportPresets,
		importPresets,
		loadFromStorage,
		clearLogs,
	};
});
