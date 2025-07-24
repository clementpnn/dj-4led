import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { EFFECTS } from '../config';
import type { Effect, EffectInfo, EffectState } from '../types';

export const useEffectsStore = defineStore('effects', () => {
	// ===== STATE =====

	const availableEffects = ref<Effect[]>([...EFFECTS]);
	const currentEffect = ref<EffectState | null>(null);
	const effectInfo = ref<EffectInfo | null>(null);
	const loading = ref(false);

	// ===== GETTERS =====

	const currentEffectName = computed(() => currentEffect.value?.name || 'None');

	const isTransitioning = computed(() => currentEffect.value?.transitioning || false);

	const transitionProgress = computed(() => currentEffect.value?.transition_progress || 0);

	const effectsByCategory = computed(() => {
		const categories: Record<string, Effect[]> = {};
		availableEffects.value.forEach((effect) => {
			const category = effect.category || 'other';
			if (!categories[category]) {
				categories[category] = [];
			}
			categories[category].push(effect);
		});
		return categories;
	});

	const getEffectById = computed(() => (id: number) => availableEffects.value.find((effect) => effect.id === id));

	// ===== ACTIONS =====

	const setAvailableEffects = (effects: Effect[]) => {
		availableEffects.value = effects;
	};

	const setCurrentEffect = (effect: EffectState | null) => {
		currentEffect.value = effect;
	};

	const setEffectInfo = (info: EffectInfo | null) => {
		effectInfo.value = info;
	};

	const setLoading = (isLoading: boolean) => {
		loading.value = isLoading;
	};

	const updateTransition = (progress: number) => {
		if (currentEffect.value) {
			currentEffect.value.transition_progress = progress;
			currentEffect.value.transitioning = progress < 1;
		}
	};

	const startTransition = (effectId: number, effectName: string) => {
		currentEffect.value = {
			id: effectId,
			name: effectName,
			transitioning: true,
			transition_progress: 0,
		};
	};

	const completeTransition = () => {
		if (currentEffect.value) {
			currentEffect.value.transitioning = false;
			currentEffect.value.transition_progress = 1;
		}
	};

	const reset = () => {
		currentEffect.value = null;
		effectInfo.value = null;
		loading.value = false;
		availableEffects.value = [...EFFECTS];
	};

	return {
		// State
		availableEffects,
		currentEffect,
		effectInfo,
		loading,

		// Getters
		currentEffectName,
		isTransitioning,
		transitionProgress,
		effectsByCategory,
		getEffectById,

		// Actions
		setAvailableEffects,
		setCurrentEffect,
		setEffectInfo,
		setLoading,
		updateTransition,
		startTransition,
		completeTransition,
		reset,
	};
});
