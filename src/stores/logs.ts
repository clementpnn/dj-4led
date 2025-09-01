// stores/logs.ts
import { defineStore } from 'pinia';
import { computed, readonly, ref } from 'vue';

import { APP_CONFIG } from '@/config';
import type { LogEntry, LogFilter, LogStats } from '@/types';

export const useLogsStore = defineStore('logs', () => {
	// ===== STATE =====
	const logs = ref<LogEntry[]>([]);
	const filter = ref<LogFilter>({
		types: ['info', 'success', 'error', 'warning'],
		categories: [],
		searchText: '',
	});
	const maxLogs = ref<number>(APP_CONFIG.performance.maxLogs);

	// ===== GETTERS =====
	const filteredLogs = computed(() => {
		let filtered = logs.value;

		// Filter by type
		if (filter.value.types.length > 0) {
			filtered = filtered.filter((log) => filter.value.types.includes(log.type));
		}

		// Filter by category
		if (filter.value.categories.length > 0) {
			filtered = filtered.filter((log) => log.category && filter.value.categories.includes(log.category));
		}

		// Filter by search text
		if (filter.value.searchText) {
			const searchLower = filter.value.searchText.toLowerCase();
			filtered = filtered.filter((log) => log.message.toLowerCase().includes(searchLower));
		}

		// Filter by time range
		if (filter.value.timeRange) {
			filtered = filtered.filter(
				(log) => log.timestamp >= filter.value.timeRange!.start && log.timestamp <= filter.value.timeRange!.end
			);
		}

		return filtered;
	});

	const logStats = computed((): LogStats => {
		const stats: LogStats = {
			total: logs.value.length,
			byType: {
				info: 0,
				success: 0,
				error: 0,
				warning: 0,
				debug: 0,
			},
			byCategory: {},
			timeRange: {
				oldest: 0,
				newest: 0,
			},
		};

		if (logs.value.length > 0) {
			// Count by type
			logs.value.forEach((log) => {
				stats.byType[log.type]++;

				// Count by category
				if (log.category) {
					stats.byCategory[log.category] = (stats.byCategory[log.category] || 0) + 1;
				}
			});

			// Time range
			const timestamps = logs.value.map((log) => log.timestamp);
			stats.timeRange.oldest = Math.min(...timestamps);
			stats.timeRange.newest = Math.max(...timestamps);
		}

		return stats;
	});

	const recentErrors = computed(() =>
		logs.value
			.filter((log) => log.type === 'error')
			.slice(-5)
			.reverse()
	);

	const recentWarnings = computed(() =>
		logs.value
			.filter((log) => log.type === 'warning')
			.slice(-5)
			.reverse()
	);

	// ===== ACTIONS =====
	const addLog = (
		message: string,
		type: LogEntry['type'] = 'info',
		category?: LogEntry['category'],
		details?: any
	) => {
		const logEntry: LogEntry = {
			id: `log-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
			time: new Date().toLocaleTimeString(),
			timestamp: Date.now(),
			message,
			type,
			category,
			details,
		};

		logs.value.push(logEntry);

		// Limit logs to prevent memory issues
		if (logs.value.length > maxLogs.value) {
			logs.value = logs.value.slice(-maxLogs.value);
		}
	};

	const clearLogs = () => {
		logs.value.splice(0, logs.value.length);
	};

	const setFilter = (newFilter: Partial<LogFilter>) => {
		Object.assign(filter.value, newFilter);
	};

	const setMaxLogs = (max: number) => {
		maxLogs.value = Math.max(10, Math.min(1000, max));

		// Trim existing logs if needed
		if (logs.value.length > maxLogs.value) {
			logs.value = logs.value.slice(-maxLogs.value);
		}
	};

	const exportLogs = (filtered = false): string => {
		const logsToExport = filtered ? filteredLogs.value : logs.value;
		return JSON.stringify(
			{
				version: '2.0.0',
				exported: Date.now(),
				count: logsToExport.length,
				logs: logsToExport,
			},
			null,
			2
		);
	};

	const initLogs = () => {
		addLog('🎵 DJ-4LED Controller ready!', 'info', 'system');
		addLog('📡 Backend connection established', 'success', 'system');
	};

	const reset = () => {
		logs.value.splice(0, logs.value.length);
		filter.value = {
			types: ['info', 'success', 'error', 'warning'],
			categories: [],
			searchText: '',
		};
		maxLogs.value = APP_CONFIG.performance.maxLogs;
	};

	return {
		// State
		logs: readonly(logs),
		filter: readonly(filter),
		maxLogs: readonly(maxLogs),

		// Getters
		filteredLogs,
		logStats,
		recentErrors,
		recentWarnings,

		// Actions
		addLog,
		clearLogs,
		setFilter,
		setMaxLogs,
		exportLogs,
		initLogs,
		reset,
	};
});
