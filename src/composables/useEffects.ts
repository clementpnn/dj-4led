import { invoke } from '@tauri-apps/api/core';
import { useEffectsStore } from '../stores/effects';
import { useLogsStore } from '../stores/logs';
import type { ActionResult, EffectInfo } from '../types';

export function useEffects() {
	const effectsStore = useEffectsStore();
	const logsStore = useLogsStore();

	// Récupérer les effets disponibles
	const getAvailableEffects = async (): Promise<ActionResult> => {
		effectsStore.setLoading(true);
		try {
			const effects = await invoke<any[]>('get_available_effects');
			effectsStore.setAvailableEffects(effects);
			logsStore.addLog(`🎇 Loaded ${effects.length} effects`, 'success', 'effects');
			return { success: true, message: `Loaded ${effects.length} effects`, data: effects };
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			logsStore.addLog(`Failed to get available effects: ${errorMessage}`, 'error', 'effects');
			return { success: false, message: `Failed to get available effects: ${errorMessage}` };
		} finally {
			effectsStore.setLoading(false);
		}
	};

	// Définir un effet par ID
	const setEffect = async (effectId: number): Promise<ActionResult> => {
		effectsStore.setLoading(true);
		try {
			const result = await invoke<string>('set_effect', { effectId });

			// Récupérer l'état actuel après changement
			await getCurrentEffect();

			const effectName = effectsStore.getEffectById(effectId)?.display_name || `Effect ${effectId}`;
			logsStore.addLog(`🎇 Effect changed to: ${effectName}`, 'success', 'effects');
			return { success: true, message: result };
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			logsStore.addLog(`Failed to set effect ${effectId}: ${errorMessage}`, 'error', 'effects');
			return { success: false, message: `❌ Effect error: ${errorMessage}` };
		} finally {
			effectsStore.setLoading(false);
		}
	};

	// Définir un effet par nom
	const setEffectByName = async (effectName: string): Promise<ActionResult> => {
		effectsStore.setLoading(true);
		try {
			const result = await invoke<any>('set_effect_by_name', { effectName });
			effectsStore.setCurrentEffect({
				id: result.id,
				name: result.name,
				transitioning: false,
				transition_progress: 0,
			});
			logsStore.addLog(`🎇 Effect changed to: ${result.name}`, 'success', 'effects');
			return {
				success: true,
				message: result.message,
				data: { id: result.id, name: result.name },
			};
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			logsStore.addLog(`Failed to set effect by name "${effectName}": ${errorMessage}`, 'error', 'effects');
			return { success: false, message: `❌ Effect error: ${errorMessage}` };
		} finally {
			effectsStore.setLoading(false);
		}
	};

	// Récupérer l'effet actuel
	const getCurrentEffect = async (): Promise<ActionResult> => {
		try {
			const effect = await invoke<any>('get_current_effect');
			effectsStore.setCurrentEffect(effect);
			return { success: true, message: 'Current effect retrieved', data: effect };
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			logsStore.addLog(`Failed to get current effect: ${errorMessage}`, 'warning', 'effects');
			return { success: false, message: `Failed to get current effect: ${errorMessage}` };
		}
	};

	// Récupérer les statistiques des effets
	const getEffectStats = async (): Promise<ActionResult> => {
		try {
			const stats = await invoke<any>('get_effect_stats');
			return { success: true, message: 'Effect stats retrieved', data: stats };
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			logsStore.addLog(`Failed to get effect stats: ${errorMessage}`, 'warning', 'effects');
			return { success: false, message: `Failed to get effect stats: ${errorMessage}` };
		}
	};

	// Récupérer les informations d'un effet
	const getEffectInfo = async (effectId: number): Promise<ActionResult> => {
		try {
			const info = await invoke<EffectInfo>('get_effect_info', { effectId });
			effectsStore.setEffectInfo(info);
			logsStore.addLog(`ℹ️ Effect info: ${info.name} - ${info.description}`, 'info', 'effects');
			return { success: true, message: 'Effect info retrieved', data: info };
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			logsStore.addLog(`Failed to get effect info for ${effectId}: ${errorMessage}`, 'warning', 'effects');
			return { success: false, message: `Failed to get effect info: ${errorMessage}` };
		}
	};

	return {
		// Store state access
		availableEffects: effectsStore.availableEffects,
		currentEffect: effectsStore.currentEffect,
		effectInfo: effectsStore.effectInfo,
		loading: effectsStore.loading,
		currentEffectName: effectsStore.currentEffectName,
		isTransitioning: effectsStore.isTransitioning,
		transitionProgress: effectsStore.transitionProgress,
		effectsByCategory: effectsStore.effectsByCategory,
		getEffectById: effectsStore.getEffectById,

		// Actions
		getAvailableEffects,
		setEffect,
		setEffectByName,
		getCurrentEffect,
		getEffectStats,
		getEffectInfo,
		reset: effectsStore.reset,
	};
}
