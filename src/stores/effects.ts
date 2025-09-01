import { defineStore } from 'pinia';
import { computed, ref, watch } from 'vue';

import { EFFECTS } from '@/config';
import type { Effect, EffectInfo, EffectState } from '@/types';

export const useEffectsStore = defineStore('effects', () => {
	// ===== STATE =====
	const availableEffects = ref<Effect[]>([...EFFECTS]);
	const currentEffect = ref<EffectState | null>(null);
	const effectInfo = ref<EffectInfo | null>(null);
	const loading = ref(false);

	// Force reactivity with watchers
	watch(currentEffect, (newEffect) => {
		console.log(`🎇 [EFFECTS_STORE] Current effect changed:`, newEffect);
	});

	watch(availableEffects, (newEffects) => {
		console.log(`🎇 [EFFECTS_STORE] Available effects updated: ${newEffects.length} effects`);
	});

	// ===== GETTERS =====
	const currentEffectName = computed(() => {
		return currentEffect.value?.name || 'None';
	});

	const isTransitioning = computed(() => {
		return currentEffect.value?.transitioning || false;
	});

	const transitionProgress = computed(() => {
		return currentEffect.value?.transition_progress || 0;
	});

	const getEffectById = computed(() => (id: number) => {
		return availableEffects.value.find((effect) => effect.id === id);
	});

	const getEffectByName = computed(() => (name: string) => {
		return availableEffects.value.find((effect) => effect.name === name);
	});

	// ===== ACTIONS =====
	const setAvailableEffects = (effects: Effect[]) => {
		console.log(`🎇 [EFFECTS_STORE] Setting available effects: ${effects.length} effects`);
		availableEffects.value = [...effects];
	};

	const setCurrentEffect = (effect: EffectState | null) => {
		console.log(`🎇 [EFFECTS_STORE] Setting current effect:`, {
			from: currentEffect.value,
			to: effect,
		});
		currentEffect.value = effect ? { ...effect } : null;
	};

	const setEffectInfo = (info: EffectInfo | null) => {
		console.log(`ℹ️ [EFFECTS_STORE] Setting effect info:`, info);
		effectInfo.value = info ? { ...info } : null;
	};

	const setLoading = (isLoading: boolean) => {
		console.log(`⏳ [EFFECTS_STORE] Loading: ${loading.value} → ${isLoading}`);
		loading.value = isLoading;
	};

	const startTransition = (effectId: number, effectName: string) => {
		console.log(`🔄 [EFFECTS_STORE] Starting transition: ${effectName} (ID: ${effectId})`);
		setCurrentEffect({
			id: effectId,
			name: effectName,
			transitioning: true,
			transition_progress: 0,
		});
	};

	const completeTransition = () => {
		if (currentEffect.value) {
			console.log(`✅ [EFFECTS_STORE] Completing transition for: ${currentEffect.value.name}`);
			const updated = {
				...currentEffect.value,
				transitioning: false,
				transition_progress: 1,
			};
			setCurrentEffect(updated);
		}
	};

	const validateEffectId = (id: number): boolean => {
		const isValid = availableEffects.value.some((effect) => effect.id === id);
		console.log(`🔍 [EFFECTS_STORE] Validating effect ID ${id}: ${isValid}`);
		return isValid;
	};

	// Force trigger reactivity
	const forceUpdate = () => {
		if (currentEffect.value) {
			const current = { ...currentEffect.value };
			currentEffect.value = { ...current };
		}
	};

	// Ensure default state
	const ensureDefaultEffect = () => {
		if (!currentEffect.value) {
			console.log(`🎇 [EFFECTS_STORE] No current effect, setting default SpectrumBars`);
			setCurrentEffect({
				id: 0,
				name: 'SpectrumBars',
				transitioning: false,
				transition_progress: 1,
			});
		}
	};

	// Pinia $reset method
	const $reset = () => {
		console.log('🔄 [EFFECTS_STORE] Resetting store');
		availableEffects.value = [...EFFECTS];
		currentEffect.value = null;
		effectInfo.value = null;
		loading.value = false;
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
		getEffectById,
		getEffectByName,

		// Actions
		setAvailableEffects,
		setCurrentEffect,
		setEffectInfo,
		setLoading,
		startTransition,
		completeTransition,
		validateEffectId,
		forceUpdate,
		ensureDefaultEffect,
		$reset,
	};
});
