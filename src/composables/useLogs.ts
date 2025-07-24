import { nextTick, ref } from 'vue';
import { useLogsStore } from '../stores/logs';
import type { ActionResult, LogEntry } from '../types';

export function useLogs() {
	const logsStore = useLogsStore();
	const logContainer = ref<HTMLElement | undefined>(undefined);
	const autoScroll = ref(true);

	// Ajouter un log simple
	const log = (message: string, type: LogEntry['type'] = 'info', category?: LogEntry['category']): void => {
		logsStore.addLog(message, type, category);
		handleAutoScroll();
	};

	// Auto-scroll
	const handleAutoScroll = (): void => {
		if (logContainer.value && autoScroll.value) {
			nextTick(() => {
				if (logContainer.value) {
					logContainer.value.scrollTop = logContainer.value.scrollHeight;
				}
			});
		}
	};

	// Toggle auto-scroll
	const toggleAutoScroll = (): void => {
		autoScroll.value = !autoScroll.value;
		if (autoScroll.value) {
			handleAutoScroll();
		}
	};

	// Logs d'initialisation
	const initLogs = (): void => {
		logsStore.initLogs();
		handleAutoScroll();
	};

	// Log pour les actions avec emoji automatique
	const logAction = (
		action: string,
		result: { success: boolean; message: string },
		category?: LogEntry['category']
	): void => {
		const type = result.success ? 'success' : 'error';
		const emoji = result.success ? '✅' : '❌';
		log(`${emoji} ${action}: ${result.message}`, type, category);
	};

	// Export simple en JSON
	const exportLogs = (): ActionResult => {
		try {
			const data = logsStore.exportLogs(false);
			const blob = new Blob([data], { type: 'application/json' });
			const url = URL.createObjectURL(blob);
			const a = document.createElement('a');
			a.href = url;
			a.download = `dj4led-logs-${new Date().toISOString().split('T')[0]}.json`;
			document.body.appendChild(a);
			a.click();
			document.body.removeChild(a);
			URL.revokeObjectURL(url);

			log('📤 Logs exported successfully', 'success', 'user');
			return { success: true, message: 'Logs exported successfully' };
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			log(`Failed to export logs: ${errorMessage}`, 'error', 'user');
			return { success: false, message: `Failed to export logs: ${errorMessage}` };
		}
	};

	// Filtrer par type
	const filterByType = (types: LogEntry['type'][]): void => {
		logsStore.setFilter({ types });
	};

	// Rechercher
	const search = (searchText: string): void => {
		logsStore.setFilter({ searchText });
	};

	return {
		// Store state
		logs: logsStore.logs,
		filteredLogs: logsStore.filteredLogs,
		logStats: logsStore.logStats,
		recentErrors: logsStore.recentErrors,
		recentWarnings: logsStore.recentWarnings,

		// Local state
		logContainer,
		autoScroll,

		// Actions
		log,
		logAction,
		toggleAutoScroll,
		exportLogs,
		filterByType,
		search,
		clearLogs: logsStore.clearLogs,
		initLogs,
		reset: logsStore.reset,
	};
}
