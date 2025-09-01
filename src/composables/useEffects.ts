import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { onMounted, onUnmounted } from 'vue';

import { useEffectsStore } from '@/stores/effects';
import type { ActionResult, EffectState } from '@/types';

export function useEffects() {
	const store = useEffectsStore();

	let unlistenEffectChanged: UnlistenFn | null = null;
	let unlistenEffectsReset: UnlistenFn | null = null;

	// ===== API CALLS =====
	const getEffectsList = async (): Promise<ActionResult> => {
		store.setLoading(true);
		try {
			const result = await invoke<any>('effects_get_list');
			const effects = result.effects || [];
			store.setAvailableEffects(effects);
			console.log(`🎇 Loaded ${effects.length} effects`);
			return {
				success: true,
				message: `Successfully loaded ${effects.length} effects`,
				data: effects,
			};
		} catch (err) {
			console.error(`❌ Failed to get effects list: ${err}`);
			return { success: false, message: String(err) };
		} finally {
			store.setLoading(false);
		}
	};

	const setEffect = async (effectId: number): Promise<ActionResult> => {
		if (!store.validateEffectId(effectId)) {
			return {
				success: false,
				message: `Invalid effect ID: ${effectId}. Valid range: 0-7`,
			};
		}

		// Prevent multiple calls
		if (store.loading) {
			return {
				success: false,
				message: 'Effect change already in progress',
			};
		}

		store.setLoading(true);
		try {
			const effect = store.getEffectById(effectId);
			if (effect) {
				store.startTransition(effectId, effect.name);
				console.log(`🎇 Starting transition to: ${effect.name} (ID: ${effectId})`);
			}

			const result = await invoke<any>('effects_set_current', { effectId });
			console.log(`✅ Effect changed to: ${result.name} (ID: ${result.id})`);

			return {
				success: true,
				message: result.message || `Effect changed to ${result.name}`,
				data: result,
			};
		} catch (err) {
			console.error(`❌ Failed to set effect ${effectId}: ${err}`);
			store.completeTransition();
			return { success: false, message: String(err) };
		} finally {
			store.setLoading(false);
		}
	};

	const setEffectByName = async (effectName: string): Promise<ActionResult> => {
		const effect = store.getEffectByName(effectName);
		if (!effect) {
			const availableNames = store.availableEffects.map((e) => e.name).join(', ');
			return {
				success: false,
				message: `Effect "${effectName}" not found. Available: ${availableNames}`,
			};
		}

		return await setEffect(effect.id);
	};

	const getCurrentEffect = async (): Promise<ActionResult> => {
		try {
			const effect = await invoke<any>('effects_get_current');
			const effectState: EffectState = {
				id: effect.id,
				name: effect.name,
				transitioning: effect.transitioning || false,
				transition_progress: effect.transition_progress || 1,
			};
			store.setCurrentEffect(effectState);
			console.log(`📊 Current effect: ${effect.name} (ID: ${effect.id})`);
			return {
				success: true,
				message: 'Current effect retrieved successfully',
				data: effect,
			};
		} catch (err) {
			console.error(`❌ Failed to get current effect: ${err}`);
			return { success: false, message: String(err) };
		}
	};

	const getEffectInfo = async (effectId: number): Promise<ActionResult> => {
		if (effectId < 0 || effectId > 7) {
			return {
				success: false,
				message: `Invalid effect ID: ${effectId}. Valid range: 0-7`,
			};
		}

		try {
			const info = await invoke<any>('effects_get_info', { effectId });
			store.setEffectInfo({
				id: info.id,
				name: info.name,
				description: info.description,
				performance_impact: info.performance_impact,
				supports_transitions: info.supports_transitions || false,
				supports_custom_colors: info.supports_custom_colors || false,
			});
			console.log(`ℹ️ Effect info loaded: ${info.name}`);
			return {
				success: true,
				message: `Effect info loaded for ${info.name}`,
				data: info,
			};
		} catch (err) {
			console.error(`❌ Failed to get effect info: ${err}`);
			return { success: false, message: String(err) };
		}
	};

	const getEffectStats = async (): Promise<ActionResult> => {
		try {
			const stats = await invoke<any>('effects_get_stats');

			// Update store with stats
			if (stats.current_effect) {
				const effectState: EffectState = {
					id: stats.current_effect.id,
					name: stats.current_effect.name,
					transitioning: stats.transition?.active || false,
					transition_progress: stats.transition?.progress || 1,
				};
				store.setCurrentEffect(effectState);
			}

			console.log(`📊 Effect stats retrieved`);
			return {
				success: true,
				message: 'Effect stats retrieved successfully',
				data: stats,
			};
		} catch (err) {
			console.error(`❌ Failed to get effect stats: ${err}`);
			return { success: false, message: String(err) };
		}
	};

	const resetAllEffects = async (): Promise<ActionResult> => {
		store.setLoading(true);
		try {
			const result = await invoke<any>('effects_reset_all');
			store.setCurrentEffect(null);
			console.log(`🔄 All effects reset`);
			return {
				success: true,
				message: result.message || 'All effects reset successfully',
				data: result,
			};
		} catch (err) {
			console.error(`❌ Failed to reset effects: ${err}`);
			return { success: false, message: String(err) };
		} finally {
			store.setLoading(false);
		}
	};

	// ===== EVENT HANDLERS =====
	const handleEffectChanged = (event: any) => {
		const data = event.payload;
		console.log(`🎇 [EVENT] Effect changed:`, data);

		if (data.id !== undefined && data.name) {
			store.setCurrentEffect({
				id: data.id,
				name: data.name,
				transitioning: false,
				transition_progress: 1,
			});
			store.completeTransition();
		}
	};

	const handleEffectsReset = (event: any) => {
		console.log(`🔄 [EVENT] Effects reset:`, event.payload);
		store.setCurrentEffect(null);
		// Refresh current state from backend
		getCurrentEffect().catch((err) => {
			console.warn('Failed to refresh effect after reset:', err);
		});
	};

	// ===== LIFECYCLE =====
	const setupListeners = async (): Promise<void> => {
		try {
			console.log(`🔧 Setting up effects event listeners...`);
			unlistenEffectChanged = await listen('effect_changed', handleEffectChanged);
			unlistenEffectsReset = await listen('effects_reset', handleEffectsReset);
			console.log(`✅ Effects event listeners setup complete`);
		} catch (err) {
			console.error('❌ Failed to setup effects listeners:', err);
		}
	};

	const cleanup = (): void => {
		console.log(`🧹 Cleaning up effects listeners...`);
		unlistenEffectChanged?.();
		unlistenEffectsReset?.();
		unlistenEffectChanged = null;
		unlistenEffectsReset = null;
	};

	const initialize = async (): Promise<void> => {
		console.log('🎇 Initializing effects composable...');
		try {
			await setupListeners();

			// Get initial state from backend
			const [effectsResult, currentResult] = await Promise.allSettled([getEffectsList(), getCurrentEffect()]);

			if (effectsResult.status === 'fulfilled' && effectsResult.value.success) {
				console.log('✅ Effects list initialized');
			}
			if (currentResult.status === 'fulfilled' && currentResult.value.success) {
				console.log('✅ Current effect initialized');
			}

			console.log('✅ Effects composable initialized');
		} catch (err) {
			console.error('❌ Failed to initialize effects:', err);
		}
	};

	// ===== UTILITIES =====
	const syncWithBackend = async (): Promise<ActionResult> => {
		console.log('🔄 Syncing effects with backend...');
		try {
			await Promise.all([getEffectsList(), getCurrentEffect()]);
			return {
				success: true,
				message: 'Effects synchronized with backend successfully',
			};
		} catch (err) {
			return {
				success: false,
				message: `Failed to sync: ${String(err)}`,
			};
		}
	};

	onMounted(() => {
		console.log('🎇 Effects composable mounted');
		initialize();
	});

	onUnmounted(() => {
		console.log('💀 Effects composable unmounting');
		cleanup();
	});

	return {
		// Store
		...store,

		// Methods
		getEffectsList,
		setEffect,
		setEffectByName,
		getCurrentEffect,
		getEffectInfo,
		getEffectStats,
		resetAllEffects,
		initialize,
		syncWithBackend,
	};
}
