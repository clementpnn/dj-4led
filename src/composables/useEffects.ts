// composables/useEffects.ts
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { onMounted, onUnmounted } from 'vue';
import { useEffectsStore } from '../stores/effects';
import type { ActionResult, Effect, EffectInfo, EffectState } from '../types';

export function useEffects() {
	// Store instance
	const effectsStore = useEffectsStore();

	// Event listeners references
	let unlistenEffectChanged: UnlistenFn | null = null;
	let unlistenEffectsReset: UnlistenFn | null = null;

	// ===== EFFECTS LIST ACTIONS =====

	const getEffectsList = async (): Promise<ActionResult> => {
		effectsStore.setLoading(true);

		try {
			const result = await invoke<any>('effects_get_list');
			const effects = result.effects || [];
			effectsStore.setAvailableEffects(effects);

			console.log(`🎇 Loaded ${effects.length} effects:`, result);
			return {
				success: true,
				message: `Loaded ${effects.length} effects`,
				data: effects,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			console.error('❌ Failed to get effects list:', errorMessage);
			return { success: false, message: errorMessage };
		} finally {
			effectsStore.setLoading(false);
		}
	};

	// ===== EFFECT SELECTION ACTIONS =====

	const setEffect = async (effectId: number): Promise<ActionResult> => {
		effectsStore.setLoading(true);

		try {
			// Start transition in store
			const effect = effectsStore.getEffectById(effectId);
			if (effect) {
				effectsStore.startTransition(effectId, effect.name);
			}

			const result = await invoke<any>('effects_set_current', { effectId });

			console.log(`🎇 Effect changed to ID ${effectId}:`, result);
			return {
				success: true,
				message: result.message || `Effect changed to ${result.name}`,
				data: result,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			console.error(`❌ Failed to set effect ${effectId}:`, errorMessage);
			return { success: false, message: errorMessage };
		} finally {
			effectsStore.setLoading(false);
		}
	};

	const setEffectByName = async (effectName: string): Promise<ActionResult> => {
		effectsStore.setLoading(true);

		try {
			const result = await invoke<any>('effects_set_by_name', { effectName });

			// Update store with result
			if (result.id !== undefined) {
				effectsStore.setCurrentEffect({
					id: result.id,
					name: result.name,
					transitioning: false,
					transition_progress: 1,
				});
			}

			console.log(`🎇 Effect changed to "${effectName}":`, result);
			return {
				success: true,
				message: result.message || `Effect changed to ${effectName}`,
				data: { id: result.id, name: result.name },
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			console.error(`❌ Failed to set effect "${effectName}":`, errorMessage);
			return { success: false, message: errorMessage };
		} finally {
			effectsStore.setLoading(false);
		}
	};

	// ===== STATUS & INFO ACTIONS =====

	const getCurrentEffect = async (): Promise<ActionResult> => {
		try {
			const effect = await invoke<any>('effects_get_current');

			const effectState: EffectState = {
				id: effect.id,
				name: effect.name,
				transitioning: effect.transitioning || false,
				transition_progress: effect.transition_progress || 1,
			};

			effectsStore.setCurrentEffect(effectState);

			console.log('📊 Current effect:', effect);
			return {
				success: true,
				message: 'Current effect retrieved',
				data: effect,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			console.warn('Failed to get current effect:', errorMessage);
			return { success: false, message: errorMessage };
		}
	};

	const getEffectInfo = async (effectId: number): Promise<ActionResult> => {
		try {
			const info = await invoke<any>('effects_get_info', { effectId });

			const effectInfo: EffectInfo = {
				id: info.id,
				name: info.name,
				description: info.description,
				performance_impact: info.performance_impact,
				supports_transitions: info.supports_transitions || false,
				supports_custom_colors: info.supports_custom_colors || false,
			};

			effectsStore.setEffectInfo(effectInfo);

			console.log(`ℹ️ Effect info for ID ${effectId}:`, info);
			return {
				success: true,
				message: 'Effect info retrieved',
				data: info,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			console.warn(`Failed to get effect info for ${effectId}:`, errorMessage);
			return { success: false, message: errorMessage };
		}
	};

	const getEffectStats = async (): Promise<ActionResult> => {
		try {
			const stats = await invoke<any>('effects_get_stats');

			// Update store with stats
			if (stats.current_effect) {
				effectsStore.setCurrentEffect({
					id: stats.current_effect.id,
					name: stats.current_effect.name,
					transitioning: stats.transition?.active || false,
					transition_progress: stats.transition?.progress || 1,
				});
			}

			console.log('📊 Effect stats:', stats);
			return {
				success: true,
				message: 'Effect stats retrieved',
				data: stats,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			console.warn('Failed to get effect stats:', errorMessage);
			return { success: false, message: errorMessage };
		}
	};

	// ===== UTILITY ACTIONS =====

	const resetAllEffects = async (): Promise<ActionResult> => {
		effectsStore.setLoading(true);

		try {
			const result = await invoke<any>('effects_reset_all');

			// Reset current effect in store
			effectsStore.setCurrentEffect(null);

			console.log('🔄 All effects reset:', result);
			return {
				success: true,
				message: result.message || 'All effects reset',
				data: result,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			console.error('❌ Failed to reset effects:', errorMessage);
			return { success: false, message: errorMessage };
		} finally {
			effectsStore.setLoading(false);
		}
	};

	// ===== HELPER FUNCTIONS =====

	const getEffectById = (id: number): Effect | undefined => {
		return effectsStore.getEffectById(id);
	};

	const getEffectByName = (name: string): Effect | undefined => {
		return effectsStore.availableEffects.find((effect) => effect.name === name);
	};

	// ===== EVENT HANDLERS =====

	const handleEffectChanged = (event: any) => {
		const data = event.payload;

		const effectState: EffectState = {
			id: data.id,
			name: data.name,
			transitioning: false,
			transition_progress: 1,
		};

		effectsStore.setCurrentEffect(effectState);
		effectsStore.completeTransition();

		console.log('🎇 Effect changed event:', data);
	};

	const handleEffectsReset = () => {
		console.log('🔄 Effects reset event received');
		effectsStore.setCurrentEffect(null);
		getCurrentEffect(); // Refresh current effect
	};

	// ===== EVENT LISTENERS SETUP =====

	const setupListeners = async (): Promise<void> => {
		try {
			unlistenEffectChanged = await listen('effect_changed', handleEffectChanged);
			unlistenEffectsReset = await listen('effects_reset', handleEffectsReset);

			console.log('✅ Effects event listeners setup complete');
		} catch (err) {
			console.error('❌ Failed to setup effects event listeners:', err);
		}
	};

	const cleanup = (): void => {
		const listeners = [
			{ fn: unlistenEffectChanged, name: 'effect_changed' },
			{ fn: unlistenEffectsReset, name: 'effects_reset' },
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

		unlistenEffectChanged = null;
		unlistenEffectsReset = null;
	};

	// ===== INITIALIZATION =====

	const initialize = async (): Promise<void> => {
		console.log('🎇 Initializing effects composable...');

		try {
			await setupListeners();
			await getEffectsList();
			await getCurrentEffect();

			console.log('✅ Effects composable initialized successfully');
		} catch (err) {
			console.error('❌ Failed to initialize effects composable:', err);
		}
	};

	// ===== LIFECYCLE =====

	onMounted(() => {
		console.log('🎇 Effects composable mounted');
		initialize();
	});

	onUnmounted(() => {
		console.log('💀 Effects composable unmounting');
		cleanup();
	});

	// ===== PUBLIC API =====

	return {
		// Store access
		...effectsStore,

		// Actions
		getEffectsList,
		setEffect,
		setEffectByName,
		getCurrentEffect,
		getEffectInfo,
		getEffectStats,
		resetAllEffects,

		// Helpers
		getEffectById,
		getEffectByName,

		// Utilities
		initialize,
		cleanup,
	};
}
