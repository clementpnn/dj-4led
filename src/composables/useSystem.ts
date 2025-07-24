import { invoke } from '@tauri-apps/api/core';
import { onMounted, onUnmounted } from 'vue';
import { useLogsStore } from '../stores/logs';
import { useSystemStore } from '../stores/system';
import type { ActionResult, SystemStats } from '../types';

export function useSystem() {
	const systemStore = useSystemStore();
	const logsStore = useLogsStore();

	let monitoringInterval: number | null = null;

	// Récupérer les statistiques système
	const getSystemStats = async (): Promise<ActionResult> => {
		try {
			const [audioStats, effectStats, ledStats] = await Promise.allSettled([
				invoke<any>('get_current_spectrum')
					.then(() => ({ is_capturing: true }))
					.catch(() => ({ is_capturing: false })),
				invoke<any>('get_effect_stats').catch(() => null),
				invoke<any>('get_led_stats').catch(() => null),
			]);

			const systemStats: SystemStats = {
				audio: {
					is_capturing: audioStats.status === 'fulfilled' ? audioStats.value.is_capturing : false,
					gain: 1.0,
					spectrum_size: 64,
					device_count: 0,
				},
				effects: {
					current_effect:
						effectStats.status === 'fulfilled' && effectStats.value?.current_effect?.name
							? effectStats.value.current_effect.name
							: 'None',
					transitioning:
						effectStats.status === 'fulfilled' ? effectStats.value?.transition?.active || false : false,
					available_effects: 8,
				},
				led: {
					is_running: ledStats.status === 'fulfilled' ? ledStats.value?.is_running || false : false,
					brightness: ledStats.status === 'fulfilled' ? ledStats.value?.brightness || 1.0 : 1.0,
					controllers: ledStats.status === 'fulfilled' ? ledStats.value?.controllers || 0 : 0,
					frame_rate: 60,
				},
				performance: {
					fps: 0,
					frame_count: 0,
					uptime: Date.now(),
				},
			};

			systemStore.setStats(systemStats);
			systemStore.setOnline(true);
			return { success: true, message: 'System stats retrieved', data: systemStats };
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			systemStore.setOnline(false);
			logsStore.addLog(`Failed to get system stats: ${errorMessage}`, 'error', 'system');
			return { success: false, message: `Failed to get system stats: ${errorMessage}` };
		}
	};

	// Démarrer le monitoring
	const startMonitoring = (intervalMs: number = 3000): void => {
		if (monitoringInterval) {
			clearInterval(monitoringInterval);
		}

		systemStore.setMonitoring(true);
		monitoringInterval = window.setInterval(() => {
			getSystemStats();
			systemStore.performHealthCheck();
		}, intervalMs);

		logsStore.addLog(`📊 System monitoring started (${intervalMs}ms)`, 'info', 'system');
	};

	// Arrêter le monitoring
	const stopMonitoring = (): void => {
		if (monitoringInterval) {
			clearInterval(monitoringInterval);
			monitoringInterval = null;
		}
		systemStore.setMonitoring(false);
		logsStore.addLog('📊 System monitoring stopped', 'info', 'system');
	};

	// Test de santé simple
	const healthCheck = async (): Promise<ActionResult> => {
		try {
			await getSystemStats();
			systemStore.performHealthCheck();

			const health = systemStore.health;
			const isHealthy = health.status === 'healthy';

			if (isHealthy) {
				logsStore.addLog('✅ System health check passed', 'success', 'system');
			} else {
				logsStore.addLog(`⚠️ Health issues: ${health.issues.join(', ')}`, 'warning', 'system');
			}

			return {
				success: true,
				message: `Health: ${health.status}`,
				data: { healthy: isHealthy, issues: health.issues },
			};
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			logsStore.addLog(`Health check failed: ${errorMessage}`, 'error', 'system');
			return { success: false, message: `Health check failed: ${errorMessage}` };
		}
	};

	// Démarrage rapide
	const quickStart = async (): Promise<ActionResult> => {
		systemStore.setLoading(true);
		try {
			logsStore.addLog('🚀 Starting system...', 'info', 'system');

			// Démarrer les composants
			await invoke('start_audio_capture');
			logsStore.addLog('🎧 Audio started', 'success', 'audio');

			await invoke('start_led_output', { mode: 'simulator' });
			logsStore.addLog('💡 LED started', 'success', 'led');

			await invoke('set_effect', { effectId: 0 });
			logsStore.addLog('🎇 Effect set', 'success', 'effects');

			await getSystemStats();
			logsStore.addLog('✅ System started successfully', 'success', 'system');
			return { success: true, message: 'System started successfully' };
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			logsStore.addLog(`❌ System start failed: ${errorMessage}`, 'error', 'system');
			return { success: false, message: `Failed to start system: ${errorMessage}` };
		} finally {
			systemStore.setLoading(false);
		}
	};

	// Arrêt complet
	const shutdown = async (): Promise<ActionResult> => {
		systemStore.setLoading(true);
		try {
			logsStore.addLog('🛑 Shutting down system...', 'info', 'system');

			await Promise.all([
				invoke('stop_audio_capture').catch(() => {}),
				invoke('stop_led_output').catch(() => {}),
			]);

			stopMonitoring();
			logsStore.addLog('✅ System shut down successfully', 'success', 'system');
			return { success: true, message: 'System shut down successfully' };
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			logsStore.addLog(`❌ Shutdown failed: ${errorMessage}`, 'error', 'system');
			return { success: false, message: `Failed to shutdown: ${errorMessage}` };
		} finally {
			systemStore.setLoading(false);
		}
	};

	// Lifecycle
	onMounted(() => {
		logsStore.addLog('🖥️ System composable mounted', 'debug', 'system');
		getSystemStats();
	});

	onUnmounted(() => {
		logsStore.addLog('💀 System composable unmounting', 'debug', 'system');
		stopMonitoring();
	});

	return {
		// Store state
		stats: systemStore.stats,
		health: systemStore.health,
		isOnline: systemStore.isOnline,
		loading: systemStore.loading,
		monitoringActive: systemStore.monitoringActive,
		systemStatus: systemStore.systemStatus,
		healthScore: systemStore.healthScore,
		isHealthy: systemStore.isHealthy,
		connectionQuality: systemStore.connectionQuality,

		// Actions
		getSystemStats,
		startMonitoring,
		stopMonitoring,
		healthCheck,
		quickStart,
		shutdown,
		reset: systemStore.reset,
	};
}
