// stores/system.ts
import { defineStore } from 'pinia';
import { computed, readonly, ref } from 'vue';
import type { SystemConfig, SystemHealth, SystemStats } from '../types';

export const useSystemStore = defineStore('system', () => {
	// ===== STATE =====
	const stats = ref<SystemStats | null>(null);
	const health = ref<SystemHealth>({
		status: 'unknown',
		issues: [],
		score: 0,
		lastCheck: 0,
	});
	const config = ref<SystemConfig>({
		monitoringInterval: 5000,
		healthCheckInterval: 10000,
		autoRestart: false,
		maxMemoryUsage: 512,
		logLevel: 'info',
	});
	const performance = ref<any>({});
	const diagnostics = ref<any>({});
	const loading = ref(false);

	// ===== GETTERS =====
	const isHealthy = computed(() => health.value.status === 'healthy');

	const systemUptime = computed(() => {
		const uptime = stats.value?.performance.uptime || 0;
		const hours = Math.floor(uptime / 3600);
		const minutes = Math.floor((uptime % 3600) / 60);
		const seconds = uptime % 60;
		return `${hours}h ${minutes}m ${seconds}s`;
	});

	const overallStatus = computed(() => {
		if (!stats.value) return 'unknown';

		const issues = [];
		if (!stats.value.audio.is_capturing) issues.push('Audio not capturing');
		if (!stats.value.led.is_running) issues.push('LED not running');
		if (stats.value.performance.fps < 15) issues.push('Low FPS');

		return issues.length === 0 ? 'healthy' : issues.length === 1 ? 'warning' : 'critical';
	});

	const componentStatus = computed(() => ({
		audio: {
			status: stats.value?.audio.is_capturing ? 'running' : 'stopped',
			health: stats.value?.audio.is_capturing && stats.value.audio.spectrum_size > 0 ? 'healthy' : 'warning',
		},
		led: {
			status: stats.value?.led.is_running ? 'running' : 'stopped',
			health: stats.value?.led.is_running && stats.value.led.frame_rate > 15 ? 'healthy' : 'warning',
		},
		effects: {
			status: stats.value?.effects.current_effect ? 'active' : 'inactive',
			health: stats.value?.effects.transitioning ? 'transitioning' : 'stable',
		},
	}));

	// ===== ACTIONS =====
	const setStats = (newStats: SystemStats) => {
		stats.value = newStats;
		updateHealth();
	};

	const setPerformance = (newPerformance: any) => {
		performance.value = newPerformance;
	};

	const setDiagnostics = (newDiagnostics: any) => {
		diagnostics.value = newDiagnostics;
	};

	const setConfig = (newConfig: Partial<SystemConfig>) => {
		config.value = { ...config.value, ...newConfig };
	};

	const setLoading = (isLoading: boolean) => {
		loading.value = isLoading;
	};

	const updateHealth = () => {
		if (!stats.value) {
			health.value = {
				status: 'unknown',
				issues: ['No system data available'],
				score: 0,
				lastCheck: Date.now(),
			};
			return;
		}

		const issues: string[] = [];
		let score = 100;

		// Check audio health
		if (!stats.value.audio.is_capturing) {
			issues.push('Audio capture is not running');
			score -= 30;
		} else if (stats.value.audio.spectrum_size === 0) {
			issues.push('No audio signal detected');
			score -= 20;
		}

		// Check LED health
		if (!stats.value.led.is_running) {
			issues.push('LED output is not running');
			score -= 30;
		} else if (stats.value.led.frame_rate < 15) {
			issues.push('Low LED frame rate');
			score -= 15;
		}

		// Check effects
		if (stats.value.effects.transitioning) {
			score -= 5; // Minor impact
		}

		// Determine status
		let status: SystemHealth['status'] = 'healthy';
		if (score < 50) status = 'critical';
		else if (score < 80) status = 'warning';

		health.value = {
			status,
			issues,
			score: Math.max(0, score),
			lastCheck: Date.now(),
		};
	};

	const reset = () => {
		stats.value = null;
		health.value = {
			status: 'unknown',
			issues: [],
			score: 0,
			lastCheck: 0,
		};
		performance.value = {};
		diagnostics.value = {};
		loading.value = false;
	};

	return {
		// State
		stats: readonly(stats),
		health: readonly(health),
		config: readonly(config),
		performance: readonly(performance),
		diagnostics: readonly(diagnostics),
		loading: readonly(loading),

		// Getters
		isHealthy,
		systemUptime,
		overallStatus,
		componentStatus,

		// Actions
		setStats,
		setPerformance,
		setDiagnostics,
		setConfig,
		setLoading,
		updateHealth,
		reset,
	};
});
