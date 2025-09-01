<template>
	<div class="system-status-panel">
		<!-- Header -->
		<div class="panel-header">
			<h2>System Status</h2>
			<div class="health-indicator" :class="systemStore.health.status">
				{{ formatHealthStatus(systemStore.health.status) }}
			</div>
		</div>

		<!-- Status Grid -->
		<div class="status-grid">
			<!-- Connection -->
			<div class="status-card" :class="connectionClass">
				<div class="status-icon">📡</div>
				<div class="status-content">
					<div class="status-label">Connection</div>
					<div class="status-value">{{ connectionStatus }}</div>
					<div class="status-detail">{{ connectionDetail }}</div>
				</div>
			</div>

			<!-- Audio -->
			<div class="status-card" :class="{ active: audioStore.state.isCapturing }">
				<div class="status-icon">🎧</div>
				<div class="status-content">
					<div class="status-label">Audio Capture</div>
					<div class="status-value">{{ audioStore.state.isCapturing ? 'ACTIVE' : 'INACTIVE' }}</div>
					<div class="status-detail">Gain: {{ (audioStore.state.currentGain || 1).toFixed(1) }}x</div>
				</div>
			</div>

			<!-- LED -->
			<div class="status-card" :class="{ active: ledStore.isRunning }">
				<div class="status-icon">💡</div>
				<div class="status-content">
					<div class="status-label">LED Output</div>
					<div class="status-value">{{ ledStore.isRunning ? 'RUNNING' : 'STOPPED' }}</div>
					<div class="status-detail">{{ ledStore.controllerCount || 0 }} controllers</div>
				</div>
			</div>

			<!-- Performance -->
			<div class="status-card">
				<div class="status-icon">⚡</div>
				<div class="status-content">
					<div class="status-label">Performance</div>
					<div class="status-value">{{ framesStore.stats.fps || 0 }} FPS</div>
					<div class="status-detail">{{ systemStore.systemUptime }} uptime</div>
				</div>
			</div>
		</div>

		<!-- Health Score -->
		<div class="health-section">
			<div class="health-score">
				<span class="score-label">Health Score</span>
				<span class="score-value" :class="systemStore.health.status">{{ systemStore.health.score }}/100</span>
			</div>
			<div class="score-bar">
				<div
					class="score-fill"
					:class="systemStore.health.status"
					:style="{ width: `${systemStore.health.score}%` }"
				></div>
			</div>
		</div>

		<!-- Issues -->
		<div v-if="systemStore.health.issues.length > 0" class="issues-section">
			<div class="issues-header">Issues ({{ systemStore.health.issues.length }})</div>
			<div class="issues-list">
				<div v-for="(issue, index) in systemStore.health.issues" :key="index" class="issue-item">
					{{ issue }}
				</div>
			</div>
		</div>

		<!-- Actions -->
		<div class="controls-section">
			<button class="control-btn primary" :disabled="systemStore.loading" @click="handleHealthCheck">
				{{ systemStore.loading ? 'Checking...' : 'Health Check' }}
			</button>
			<button class="control-btn secondary" :disabled="systemStore.loading" @click="handleRefreshStats">
				Refresh
			</button>
			<button class="control-btn secondary" :disabled="systemStore.loading" @click="handleRunDiagnostics">
				Diagnostics
			</button>
		</div>
	</div>
</template>

<script setup lang="ts">
	import { computed } from 'vue';

	import { useSystem } from '@/composables/useSystem';
	import { useAudioStore } from '@/stores/audio';
	import { useFramesStore } from '@/stores/frames';
	import { useLEDStore } from '@/stores/led';
	import { useSystemStore } from '@/stores/system';

	// Props pour les données de connexion qui peuvent venir d'ailleurs
	interface Props {
		connectionQuality?: number;
		isOnline?: boolean;
	}

	const props = withDefaults(defineProps<Props>(), {
		connectionQuality: 100,
		isOnline: true,
	});

	// Stores - pour récupérer les données
	const systemStore = useSystemStore();
	const audioStore = useAudioStore();
	const ledStore = useLEDStore();
	const framesStore = useFramesStore();

	// Composables - pour la logique et les actions uniquement
	const { getStatus, runFullDiagnostics } = useSystem();

	// Computed properties basées sur les stores
	const connectionClass = computed(() => {
		if (!props.isOnline) return 'critical';
		if (props.connectionQuality >= 80) return 'healthy';
		if (props.connectionQuality >= 50) return 'warning';
		return 'critical';
	});

	const connectionStatus = computed(() => {
		return props.isOnline ? 'ONLINE' : 'OFFLINE';
	});

	const connectionDetail = computed(() => {
		return `${props.connectionQuality}% quality`;
	});

	// Handlers - utilisant les composables pour les actions
	const handleHealthCheck = async (): Promise<void> => {
		try {
			await getStatus();
			systemStore.updateHealth();
		} catch (error) {
			console.error('Failed to run health check:', error);
		}
	};

	const handleRefreshStats = async (): Promise<void> => {
		try {
			await getStatus();
		} catch (error) {
			console.error('Failed to refresh stats:', error);
		}
	};

	const handleRunDiagnostics = async (): Promise<void> => {
		try {
			await runFullDiagnostics();
		} catch (error) {
			console.error('Failed to run diagnostics:', error);
		}
	};

	// Utility functions
	const formatHealthStatus = (status: string): string => {
		return status.charAt(0).toUpperCase() + status.slice(1);
	};
</script>

<style scoped>
	.system-status-panel {
		background: #0d1117;
		border: 1px solid #30363d;
		border-radius: 8px;
		padding: 1.5rem;
		color: #c9d1d9;
	}

	/* Header */
	.panel-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1.5rem;
		padding-bottom: 0.75rem;
		border-bottom: 1px solid #21262d;
	}

	.panel-header h2 {
		margin: 0;
		font-size: 1.125rem;
		font-weight: 600;
		color: #c9d1d9;
	}

	.health-indicator {
		padding: 0.375rem 0.75rem;
		border-radius: 4px;
		font-size: 0.75rem;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.025em;
	}

	.health-indicator.healthy {
		background: #161b22;
		color: #3fb950;
		border: 1px solid #21262d;
	}

	.health-indicator.warning {
		background: #161b22;
		color: #d29922;
		border: 1px solid #21262d;
	}

	.health-indicator.critical {
		background: #161b22;
		color: #f85149;
		border: 1px solid #21262d;
	}

	.health-indicator.unknown {
		background: #161b22;
		color: #7d8590;
		border: 1px solid #21262d;
	}

	/* Status Grid */
	.status-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
		gap: 1rem;
		margin-bottom: 1.5rem;
	}

	.status-card {
		display: flex;
		align-items: center;
		gap: 1rem;
		padding: 1rem;
		border: 1px solid #21262d;
		border-radius: 4px;
		background: #161b22;
		transition: all 0.2s ease;
	}

	.status-card.active {
		border-color: #3fb950;
		background: rgba(63, 185, 80, 0.05);
	}

	.status-card.healthy {
		border-color: #3fb950;
	}

	.status-card.warning {
		border-color: #d29922;
	}

	.status-card.critical {
		border-color: #f85149;
	}

	.status-icon {
		font-size: 1.5rem;
		flex-shrink: 0;
	}

	.status-content {
		flex: 1;
	}

	.status-label {
		font-size: 0.75rem;
		color: #7d8590;
		text-transform: uppercase;
		font-weight: 500;
		margin-bottom: 0.25rem;
		letter-spacing: 0.025em;
	}

	.status-value {
		font-size: 0.875rem;
		font-weight: 600;
		color: #c9d1d9;
		margin-bottom: 0.25rem;
	}

	.status-detail {
		font-size: 0.75rem;
		color: #7d8590;
		font-family: monospace;
	}

	/* Health Score */
	.health-section {
		margin-bottom: 1.5rem;
		padding: 1rem;
		background: #161b22;
		border: 1px solid #21262d;
		border-radius: 4px;
	}

	.health-score {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 0.75rem;
	}

	.score-label {
		font-size: 0.875rem;
		font-weight: 500;
		color: #c9d1d9;
	}

	.score-value {
		font-size: 1.25rem;
		font-weight: 700;
		font-family: monospace;
	}

	.score-value.healthy {
		color: #3fb950;
	}

	.score-value.warning {
		color: #d29922;
	}

	.score-value.critical {
		color: #f85149;
	}

	.score-value.unknown {
		color: #7d8590;
	}

	.score-bar {
		height: 6px;
		background: #21262d;
		border-radius: 3px;
		overflow: hidden;
	}

	.score-fill {
		height: 100%;
		transition: width 0.5s ease;
	}

	.score-fill.healthy {
		background: #3fb950;
	}

	.score-fill.warning {
		background: #d29922;
	}

	.score-fill.critical {
		background: #f85149;
	}

	.score-fill.unknown {
		background: #7d8590;
	}

	/* Issues */
	.issues-section {
		margin-bottom: 1.5rem;
	}

	.issues-header {
		font-size: 0.875rem;
		font-weight: 600;
		color: #d29922;
		margin-bottom: 0.75rem;
	}

	.issues-list {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.issue-item {
		padding: 0.75rem;
		background: #161b22;
		border: 1px solid #21262d;
		border-radius: 4px;
		font-size: 0.875rem;
		color: #c9d1d9;
		border-left: 3px solid #d29922;
	}

	/* Controls */
	.controls-section {
		display: flex;
		gap: 0.75rem;
	}

	.control-btn {
		flex: 1;
		padding: 0.75rem 1rem;
		border: 1px solid #30363d;
		border-radius: 4px;
		font-size: 0.875rem;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.control-btn.primary {
		background: #21262d;
		color: #c9d1d9;
		border-color: #c9d1d9;
	}

	.control-btn.primary:hover:not(:disabled) {
		background: #30363d;
	}

	.control-btn.secondary {
		background: #161b22;
		color: #7d8590;
		border-color: #21262d;
	}

	.control-btn.secondary:hover:not(:disabled) {
		background: #21262d;
		color: #c9d1d9;
	}

	.control-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	/* Responsive */
	@media (max-width: 768px) {
		.status-grid {
			grid-template-columns: 1fr;
		}

		.controls-section {
			flex-direction: column;
		}
	}
</style>
