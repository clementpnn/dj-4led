// stores/presets.ts
import { defineStore } from 'pinia';
import { computed, ref } from 'vue';

import type { Preset } from '@/types';

export const usePresetsStore = defineStore('presets', () => {
	// ===== STATE =====
	const presets = ref<Preset[]>([]);
	const currentPreset = ref<Preset | null>(null);
	const loading = ref(false);
	const isApplying = ref(false);

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
	const allPresets = computed(() => {
		const combined = [...defaultPresets, ...presets.value];
		return combined;
	});

	const availableTags = computed(() => {
		const tags = new Set<string>();
		allPresets.value.forEach((preset) => {
			preset.tags?.forEach((tag) => tags.add(tag));
		});
		return Array.from(tags).sort();
	});

	// Helper function to check if a name exists (centralized logic)
	const nameExists = (name: string): boolean => {
		const normalizedName = name.toLowerCase().trim();
		return allPresets.value.some((p) => p.name.toLowerCase().trim() === normalizedName);
	};

	// ===== ACTIONS =====
	const setLoading = (value: boolean) => {
		loading.value = value;
	};

	const setApplying = (value: boolean) => {
		isApplying.value = value;
	};

	const addPreset = (preset: Preset): boolean => {
		if (nameExists(preset.name)) {
			console.warn('❌ [PRESETS_STORE] Preset name already exists:', preset.name);
			return false;
		}

		// Add to custom presets array and force reactivity update
		const newPreset = { ...preset };
		presets.value.push(newPreset);

		// Force Vue reactivity trigger
		presets.value = [...presets.value];

		// Save to localStorage
		saveToStorage();
		console.log('✅ [PRESETS_STORE] Preset added and saved successfully');
		return true;
	};

	const deletePreset = (id: string): boolean => {
		console.log(`🗑️ [PRESETS_STORE] Attempting to delete preset: ${id}`);

		// Prevent deletion of default presets
		const isDefault = defaultPresets.some((p) => p.id === id);
		if (isDefault) {
			console.warn('❌ [PRESETS_STORE] Cannot delete default preset:', id);
			return false;
		}

		const index = presets.value.findIndex((p) => p.id === id);

		if (index !== -1) {
			const deletedPreset = presets.value[index];
			presets.value.splice(index, 1);

			if (currentPreset.value?.id === id) {
				console.log('🔄 [PRESETS_STORE] Clearing current preset as it was deleted');
				currentPreset.value = null;
			}

			saveToStorage();
			console.log('✅ [PRESETS_STORE] Preset deleted successfully:', deletedPreset.name);
			return true;
		}

		console.warn('❌ [PRESETS_STORE] Preset not found in custom presets:', id);
		return false;
	};

	const setCurrentPreset = (preset: Preset | null) => {
		currentPreset.value = preset;
	};

	const duplicatePreset = (id: string, newName: string): Preset | null => {
		console.log(`🔄 [PRESETS_STORE] Duplicating preset: ${id} → ${newName}`);

		// Validate new name before proceeding
		if (nameExists(newName)) {
			return null;
		}

		const original = allPresets.value.find((p) => p.id === id);
		if (!original) {
			console.warn('❌ [PRESETS_STORE] Preset not found for duplication:', id);
			return null;
		}

		// Create deep copy to avoid reference issues
		const duplicate: Preset = {
			...original,
			id: `preset-${Date.now()}-${Math.random().toString(36).substr(2, 5)}`,
			name: newName,
			description: original.description ? `Copy of ${original.description}` : `Copy of ${original.name}`,
			createdAt: Date.now(),
			updatedAt: undefined,
			config: {
				...original.config,
				effect: { ...original.config.effect },
				color: {
					...original.config.color,
					...(original.config.color.customColor && {
						customColor: { ...original.config.color.customColor },
					}),
				},
				audio: { ...original.config.audio },
				led: { ...original.config.led },
			},
		};

		const success = addPreset(duplicate);
		if (success) {
			console.log('✅ [PRESETS_STORE] Preset duplicated successfully:', duplicate);
			return duplicate;
		} else {
			console.warn('❌ [PRESETS_STORE] Failed to add duplicated preset');
			return null;
		}
	};

	const getPresetById = (id: string): Preset | undefined => {
		return allPresets.value.find((p) => p.id === id);
	};

	const exportPresets = (includeDefaults = false): { success: boolean; message: string; data?: string } => {
		try {
			const presetsToExport = includeDefaults ? allPresets.value : presets.value;

			const data = JSON.stringify(
				{
					version: '1.0',
					exported_at: new Date().toISOString(),
					total_presets: presetsToExport.length,
					presets: presetsToExport,
				},
				null,
				2
			);

			return { success: true, message: `${presetsToExport.length} presets exported`, data };
		} catch (error) {
			return { success: false, message: 'Failed to export presets' };
		}
	};

	const importPresets = (
		data: string
	): { success: boolean; message: string; imported?: number; skipped?: number } => {
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

				// Check for duplicate names using centralized function
				if (nameExists(presetData.name)) {
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

			return {
				success: true,
				message: `${imported} presets imported${skipped > 0 ? `, ${skipped} skipped` : ''}`,
				imported,
				skipped,
			};
		} catch (error) {
			return { success: false, message: 'Failed to parse preset data' };
		}
	};

	// ===== STORAGE =====
	const saveToStorage = () => {
		try {
			const dataToSave = JSON.stringify(presets.value, null, 2);
			localStorage.setItem('dj4led-presets', dataToSave);
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
				} else {
					presets.value = [];
				}
			} else {
				presets.value = [];
			}
		} catch (error) {
			presets.value = [];
		}
	};

	const clearPresets = () => {
		presets.value.splice(0, presets.value.length);
		currentPreset.value = null;
		loading.value = false;
		isApplying.value = false;

		try {
			localStorage.removeItem('dj4led-presets');
		} catch (error) {
			console.warn('Failed to clear presets storage:', error);
		}
	};

	const isDefaultPreset = (presetId: string): boolean => {
		return defaultPresets.some((p) => p.id === presetId);
	};

	return {
		// State (reactive)
		presets,
		currentPreset,
		loading,
		isApplying,

		// Getters
		allPresets,
		availableTags,

		// Helper functions
		nameExists,

		// Actions
		setLoading,
		setApplying,
		addPreset,
		deletePreset,
		setCurrentPreset,
		duplicatePreset,
		getPresetById,
		exportPresets,
		importPresets,
		loadFromStorage,
		clearPresets,
		isDefaultPreset,
	};
});
