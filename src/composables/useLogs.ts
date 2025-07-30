// composables/useLogs.ts
import { computed, nextTick, onMounted, ref } from 'vue';
import { useLogsStore } from '../stores/logs';
import type { ActionResult, LogEntry } from '../types';

export function useLogs() {
	// Store instance
	const logsStore = useLogsStore();

	// Local UI state
	const logContainer = ref<HTMLElement | undefined>(undefined);
	const searchText = ref('');
	const selectedType = ref<string>('all');

	// ===== COMPUTED =====
	const filteredLogs = computed(() => {
		let result = logsStore.filteredLogs;

		// Additional local filtering
		if (selectedType.value !== 'all') {
			result = result.filter((log) => log.type === selectedType.value);
		}

		if (searchText.value.trim()) {
			const search = searchText.value.toLowerCase();
			result = result.filter(
				(log) =>
					log.message.toLowerCase().includes(search) ||
					(log.category && log.category.toLowerCase().includes(search))
			);
		}

		return result.slice().reverse(); // Most recent first
	});

	// ===== LOGGING ACTIONS =====
	const log = (message: string, type: LogEntry['type'] = 'info', category?: LogEntry['category']): void => {
		logsStore.addLog(message, type, category);
		handleAutoScroll();
	};

	const logAction = (
		action: string,
		result: { success: boolean; message: string },
		category?: LogEntry['category']
	): void => {
		const type = result.success ? 'success' : 'error';
		const emoji = result.success ? '✅' : '❌';
		log(`${emoji} ${action}: ${result.message}`, type, category);
	};

	// Simple helpers
	const logInfo = (message: string, category?: LogEntry['category']) => log(message, 'info', category);
	const logSuccess = (message: string, category?: LogEntry['category']) => log(message, 'success', category);
	const logWarning = (message: string, category?: LogEntry['category']) => log(message, 'warning', category);
	const logError = (message: string, category?: LogEntry['category']) => log(message, 'error', category);
	const logDebug = (message: string, category?: LogEntry['category']) => log(message, 'debug', category);

	// ===== UI ACTIONS =====
	const handleAutoScroll = (): void => {
		nextTick(() => {
			if (logContainer.value) {
				logContainer.value.scrollTop = logContainer.value.scrollHeight;
			}
		});
	};

	// ===== EXPORT =====
	const exportLogs = (): ActionResult => {
		try {
			const data = logsStore.exportLogs(false);

			const blob = new Blob([data], { type: 'application/json' });
			const url = URL.createObjectURL(blob);
			const a = document.createElement('a');
			a.href = url;
			a.download = `logs-${new Date().toISOString().split('T')[0]}.json`;
			document.body.appendChild(a);
			a.click();
			document.body.removeChild(a);
			URL.revokeObjectURL(url);

			logSuccess('Logs exported successfully', 'system');
			return { success: true, message: 'Logs exported successfully' };
		} catch (err) {
			logError('Failed to export logs', 'system');
			return { success: false, message: 'Failed to export logs' };
		}
	};

	// ===== UTILITIES =====
	const getEmoji = (type: LogEntry['type']): string => {
		const emojis = {
			debug: '🐛',
			info: 'ℹ️',
			success: '✅',
			warning: '⚠️',
			error: '❌',
		};
		return emojis[type] || 'ℹ️';
	};

	const formatTime = (timestamp: number): string => {
		return new Date(timestamp).toLocaleTimeString();
	};

	const clearLogs = (): void => {
		logsStore.clearLogs();
	};

	// ===== LIFECYCLE =====
	onMounted(() => {
		if (logsStore.logs.length === 0) {
			logsStore.initLogs();
		}
		handleAutoScroll();
	});

	// ===== PUBLIC API =====
	return {
		// Store access
		logs: logsStore.logs,
		logStats: logsStore.logStats,

		// Local state
		logContainer,
		searchText,
		selectedType,

		// Computed
		filteredLogs,

		// Actions
		log,
		logAction,
		logInfo,
		logSuccess,
		logWarning,
		logError,
		logDebug,

		// UI
		exportLogs,
		clearLogs,

		// Utils
		getEmoji,
		formatTime,
	};
}
