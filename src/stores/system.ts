import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { DEFAULT_SYSTEM_CONFIG } from '../config';
import type { SystemHealth, SystemStats } from '../types';

export const useSystemStore = defineStore('system', () => {
	// ===== STATE =====

	const stats = ref<SystemStats | null>(null);
	const health = ref<SystemHealth>({
		status: 'unknown',
		issues: [],
		score: 0,
		lastCheck: 0,
	});
	const isOnline = ref(false);
	const loading = ref(false);
	const monitoringActive = ref(false);

	// ===== GETTERS =====

	const systemStatus = computed(() => health.value.status);

	const healthScore = computed(() => health.value.score);

	const uptime = computed(() => stats.value?.performance?.uptime || 0);

	const overallFPS = computed(() => stats.value?.performance?.fps || 0);

	const memoryUsage = computed(() => stats.value?.performance?.memory_usage || 0);

	const cpuUsage = computed(() => stats.value?.performance?.cpu_usage || 0);

	const isHealthy = computed(
		() => systemStatus.value === 'healthy' && isOnline.value && health.value.issues.length === 0
	);

	const connectionQuality = computed(() => {
		if (!isOnline.value) return 0;
		if (!stats.value) return 25;

		let quality = 100;

		// Reduce quality based on issues
		if (health.value.issues.length > 0) {
			quality -= health.value.issues.length * 20;
		}

		// Check component health
		if (!stats.value.audio.is_capturing) quality -= 30;
		if (!stats.value.led.is_running) quality -= 30;

		return Math.max(0, Math.min(100, quality));
	});

	// ===== ACTIONS =====

	const setStats = (newStats: SystemStats | null) => {
		stats.value = newStats;
		isOnline.value = newStats !== null;
	};

	const updateHealth = (newHealth: Partial<SystemHealth>) => {
		health.value = { ...health.value, ...newHealth, lastCheck: Date.now() };
	};

	const setOnline = (online: boolean) => {
		isOnline.value = online;
		if (!online) {
			health.value.status = 'unknown';
			health.value.issues = ['System offline'];
		}
	};

	const setLoading = (isLoading: boolean) => {
		loading.value = isLoading;
	};

	const setMonitoring = (active: boolean) => {
		monitoringActive.value = active;
	};

	const addIssue = (issue: string) => {
		if (!health.value.issues.includes(issue)) {
			health.value.issues.push(issue);
			updateHealthStatus();
		}
	};

	const removeIssue = (issue: string) => {
		const index = health.value.issues.indexOf(issue);
		if (index > -1) {
			health.value.issues.splice(index, 1);
			updateHealthStatus();
		}
	};

	const clearIssues = () => {
		health.value.issues = [];
		updateHealthStatus();
	};

	const updateHealthStatus = () => {
		const issueCount = health.value.issues.length;

		if (!isOnline.value) {
			health.value.status = 'unknown';
			health.value.score = 0;
		} else if (issueCount === 0) {
			health.value.status = 'healthy';
			health.value.score = 100;
		} else if (issueCount <= 2) {
			health.value.status = 'warning';
			health.value.score = Math.max(50, 100 - issueCount * 25);
		} else {
			health.value.status = 'critical';
			health.value.score = Math.max(0, 100 - issueCount * 30);
		}
	};

	const performHealthCheck = () => {
		const issues: string[] = [];

		if (!stats.value) {
			issues.push('No system stats available');
		} else {
			// Check audio
			if (!stats.value.audio.is_capturing) {
				issues.push('Audio capture not active');
			}

			// Check LED
			if (!stats.value.led.is_running) {
				issues.push('LED output not running');
			}

			// Check performance
			if (stats.value.performance.fps < 10) {
				issues.push('Low frame rate detected');
			}

			// Check memory (if available)
			if (
				stats.value.performance.memory_usage &&
				stats.value.performance.memory_usage > DEFAULT_SYSTEM_CONFIG.maxMemoryUsage
			) {
				issues.push('High memory usage');
			}
		}

		health.value.issues = issues;
		updateHealthStatus();
	};

	const reset = () => {
		stats.value = null;
		health.value = {
			status: 'unknown',
			issues: [],
			score: 0,
			lastCheck: 0,
		};
		isOnline.value = false;
		loading.value = false;
		monitoringActive.value = false;
	};

	return {
		// State
		stats,
		health,
		isOnline,
		loading,
		monitoringActive,

		// Getters
		systemStatus,
		healthScore,
		uptime,
		overallFPS,
		memoryUsage,
		cpuUsage,
		isHealthy,
		connectionQuality,

		// Actions
		setStats,
		updateHealth,
		setOnline,
		setLoading,
		setMonitoring,
		addIssue,
		removeIssue,
		clearIssues,
		updateHealthStatus,
		performHealthCheck,
		reset,
	};
});
