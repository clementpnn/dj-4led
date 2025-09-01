// composables/useSystem.ts
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { onMounted, onUnmounted, ref } from 'vue';

import { useSystemStore } from '@/stores/system';
import type { ActionResult, SystemConfig, SystemStats } from '@/types';

export function useSystem() {
	// Store instance
	const systemStore = useSystemStore();

	// Local state
	const error = ref<string | null>(null);
	const monitoringInterval = ref<number | null>(null);

	// Event listeners references
	let unlistenSystemRestart: UnlistenFn | null = null;
	let unlistenConfigUpdated: UnlistenFn | null = null;

	// ===== STATUS ACTIONS =====

	const getStatus = async (): Promise<ActionResult> => {
		systemStore.setLoading(true);
		error.value = null;

		try {
			const result = await invoke<any>('system_get_status');

			// Transform backend data to SystemStats format
			const stats: SystemStats = {
				audio: {
					is_capturing: result.audio?.running || false,
					gain: result.audio?.gain || 1.0,
					spectrum_size: result.audio?.spectrum_size || 0,
					device_count: 1, // Could be enhanced from backend
				},
				effects: {
					current_effect: result.effects?.current_name || 'Unknown',
					transitioning: result.effects?.transitioning || false,
					available_effects: result.effects?.total_effects || 0,
				},
				led: {
					is_running: result.led?.running || false,
					brightness: result.led?.brightness || 1.0,
					controllers: 1, // Could be enhanced from backend
					frame_rate: result.led?.target_fps || 30,
				},
				performance: {
					fps: result.led?.target_fps || 30,
					frame_count: 0, // Could be tracked
					uptime: result.system?.uptime || 0,
				},
			};

			systemStore.setStats(stats);

			console.log('📊 System status retrieved:', result);
			return {
				success: true,
				message: 'System status retrieved',
				data: result,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			error.value = errorMessage;
			console.error('❌ Failed to get system status:', errorMessage);
			return { success: false, message: errorMessage };
		} finally {
			systemStore.setLoading(false);
		}
	};

	const getPerformance = async (): Promise<ActionResult> => {
		try {
			const result = await invoke<any>('system_get_performance');
			systemStore.setPerformance(result);

			console.log('📈 System performance retrieved:', result);
			return {
				success: true,
				message: 'Performance data retrieved',
				data: result,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			console.warn('Failed to get system performance:', errorMessage);
			return { success: false, message: errorMessage };
		}
	};

	const getDiagnostics = async (): Promise<ActionResult> => {
		try {
			const result = await invoke<any>('system_get_diagnostics');
			systemStore.setDiagnostics(result);

			console.log('🔍 System diagnostics:', result);
			return {
				success: true,
				message: 'Diagnostics completed',
				data: result,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			console.warn('Failed to get system diagnostics:', errorMessage);
			return { success: false, message: errorMessage };
		}
	};

	// ===== CONFIG ACTIONS =====

	const getConfig = async (): Promise<ActionResult> => {
		try {
			const result = await invoke<any>('system_get_config');

			// Transform to SystemConfig if needed
			const config: Partial<SystemConfig> = {
				// Map backend config to our format
				logLevel: 'info', // Default, could be from backend
				autoRestart: false, // Default, could be from backend
			};

			systemStore.setConfig(config);

			console.log('⚙️ System configuration retrieved:', result);
			return {
				success: true,
				message: 'Configuration retrieved',
				data: result,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			console.warn('Failed to get system configuration:', errorMessage);
			return { success: false, message: errorMessage };
		}
	};

	const setConfig = async (newConfig: any): Promise<ActionResult> => {
		systemStore.setLoading(true);

		try {
			const result = await invoke<any>('system_set_config', { config: newConfig });

			console.log('⚙️ System configuration updated:', result);
			return {
				success: true,
				message: result.message || 'Configuration updated successfully',
				data: result,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			error.value = errorMessage;
			console.error('❌ Failed to set system configuration:', errorMessage);
			return { success: false, message: errorMessage };
		} finally {
			systemStore.setLoading(false);
		}
	};

	// ===== SYSTEM CONTROL =====

	const restartAll = async (): Promise<ActionResult> => {
		systemStore.setLoading(true);

		try {
			const result = await invoke<any>('system_restart_all');

			console.log('🔄 System restart initiated:', result);

			// Clear local state after restart
			setTimeout(() => {
				systemStore.reset();
				error.value = null;
				getStatus(); // Refresh status after restart
			}, 1000);

			return {
				success: true,
				message: result.message || 'System restart completed',
				data: result,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			error.value = errorMessage;
			console.error('❌ Failed to restart system:', errorMessage);
			return { success: false, message: errorMessage };
		} finally {
			systemStore.setLoading(false);
		}
	};

	// ===== EXPORT/IMPORT =====

	const exportConfig = async (): Promise<ActionResult> => {
		try {
			const result = await invoke<any>('system_export_config');

			// Create downloadable file
			const blob = new Blob([JSON.stringify(result, null, 2)], {
				type: 'application/json',
			});
			const url = URL.createObjectURL(blob);
			const a = document.createElement('a');
			a.href = url;
			a.download = `system-config-${new Date().toISOString().split('T')[0]}.json`;
			document.body.appendChild(a);
			a.click();
			document.body.removeChild(a);
			URL.revokeObjectURL(url);

			console.log('📤 Configuration exported:', result);
			return {
				success: true,
				message: 'Configuration exported successfully',
				data: result,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			error.value = errorMessage;
			console.error('❌ Failed to export configuration:', errorMessage);
			return { success: false, message: errorMessage };
		}
	};

	const importConfig = async (configData: any): Promise<ActionResult> => {
		systemStore.setLoading(true);

		try {
			const result = await invoke<any>('system_import_config', { configData });

			// Refresh local state after import
			await getConfig();
			await getStatus();

			console.log('📥 Configuration imported:', result);
			return {
				success: true,
				message: result.message || 'Configuration imported successfully',
				data: result,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			error.value = errorMessage;
			console.error('❌ Failed to import configuration:', errorMessage);
			return { success: false, message: errorMessage };
		} finally {
			systemStore.setLoading(false);
		}
	};

	const importConfigFromFile = async (file: File): Promise<ActionResult> => {
		try {
			const text = await file.text();
			const configData = JSON.parse(text);
			return await importConfig(configData);
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			error.value = errorMessage;
			console.error('❌ Failed to parse configuration file:', errorMessage);
			return { success: false, message: `Invalid configuration file: ${errorMessage}` };
		}
	};

	// ===== UTILITY FUNCTIONS =====

	const runFullDiagnostics = async (): Promise<ActionResult> => {
		systemStore.setLoading(true);

		try {
			console.log('🔍 Running full system diagnostics...');

			const [statusResult, perfResult, diagResult] = await Promise.allSettled([
				getStatus(),
				getPerformance(),
				getDiagnostics(),
			]);

			const results = {
				status: statusResult.status === 'fulfilled' ? statusResult.value : null,
				performance: perfResult.status === 'fulfilled' ? perfResult.value : null,
				diagnostics: diagResult.status === 'fulfilled' ? diagResult.value : null,
			};

			const issues = [];
			if (statusResult.status === 'rejected') issues.push('Status check failed');
			if (perfResult.status === 'rejected') issues.push('Performance check failed');
			if (diagResult.status === 'rejected') issues.push('Diagnostics failed');

			const message =
				issues.length > 0
					? `Diagnostics completed with ${issues.length} issues: ${issues.join(', ')}`
					: 'Full diagnostics completed successfully';

			console.log('🔍 Full diagnostics completed:', results);
			return {
				success: issues.length === 0,
				message,
				data: results,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			error.value = errorMessage;
			console.error('❌ Full diagnostics failed:', errorMessage);
			return { success: false, message: errorMessage };
		} finally {
			systemStore.setLoading(false);
		}
	};

	const getSystemInfo = () => {
		const stats = systemStore.stats;
		if (!stats) {
			return {
				version: 'Unknown',
				name: 'LED Audio Visualizer',
				uptime: '0h 0m 0s',
				health: 'unknown',
				components: {},
			};
		}

		return {
			version: '1.0.0',
			name: 'LED Audio Visualizer',
			uptime: systemStore.systemUptime,
			health: systemStore.overallStatus,
			components: systemStore.componentStatus,
		};
	};

	// ===== MONITORING =====

	const startMonitoring = (intervalMs = 5000) => {
		if (monitoringInterval.value) {
			stopMonitoring();
		}

		monitoringInterval.value = window.setInterval(async () => {
			try {
				await getStatus();
				systemStore.updateHealth();
			} catch (err) {
				console.warn('⚠️ Monitoring update failed:', err);
			}
		}, intervalMs);

		console.log(`📊 System monitoring started (${intervalMs}ms interval)`);
	};

	const stopMonitoring = () => {
		if (monitoringInterval.value) {
			clearInterval(monitoringInterval.value);
			monitoringInterval.value = null;
			console.log('🛑 System monitoring stopped');
		}
	};

	// ===== EVENT HANDLERS =====

	const handleSystemRestart = (event: any) => {
		const data = event.payload;
		console.log('🔄 System restart event:', data);

		if (data.status === 'started') {
			systemStore.setLoading(true);
		} else if (data.status === 'completed') {
			systemStore.setLoading(false);
			getStatus(); // Refresh status after restart
		}
	};

	const handleConfigUpdated = () => {
		console.log('⚙️ Configuration updated event received');
		getConfig(); // Refresh config
	};

	// ===== EVENT LISTENERS SETUP =====

	const setupListeners = async (): Promise<void> => {
		try {
			unlistenSystemRestart = await listen('system_restart_started', handleSystemRestart);
			unlistenConfigUpdated = await listen('system_config_updated', handleConfigUpdated);

			console.log('✅ System event listeners setup complete');
		} catch (err) {
			console.error('❌ Failed to setup system event listeners:', err);
			error.value = 'Failed to setup event listeners';
		}
	};

	const cleanup = (): void => {
		const listeners = [
			{ fn: unlistenSystemRestart, name: 'system_restart_started' },
			{ fn: unlistenConfigUpdated, name: 'system_config_updated' },
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

		unlistenSystemRestart = null;
		unlistenConfigUpdated = null;

		// Stop monitoring
		stopMonitoring();
	};

	// ===== INITIALIZATION =====

	const initialize = async (): Promise<void> => {
		console.log('📊 Initializing system composable...');

		try {
			await setupListeners();
			await getStatus();
			await getConfig();

			// Start monitoring if configured
			if (systemStore.config.monitoringInterval > 0) {
				startMonitoring(systemStore.config.monitoringInterval);
			}

			console.log('✅ System composable initialized successfully');
		} catch (err) {
			console.error('❌ Failed to initialize system composable:', err);
			error.value = 'Failed to initialize system monitoring';
		}
	};

	// ===== LIFECYCLE =====

	onMounted(() => {
		console.log('📊 System composable mounted');
		initialize();
	});

	onUnmounted(() => {
		console.log('💀 System composable unmounting');
		cleanup();
	});

	// ===== PUBLIC API =====

	return {
		// Store access
		...systemStore,

		// Local state
		error,

		// Actions
		getStatus,
		getPerformance,
		getDiagnostics,
		getConfig,
		setConfig,
		restartAll,
		exportConfig,
		importConfig,
		importConfigFromFile,
		runFullDiagnostics,
		getSystemInfo,

		// Monitoring
		startMonitoring,
		stopMonitoring,

		// Utilities
		initialize,
		cleanup,
	};
}
