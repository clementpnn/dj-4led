<template>
	<div class="panel system-status-panel">
		<div class="panel-header">
			<h2>📊 System Status</h2>
			<div class="panel-subtitle">Real-time system overview</div>
		</div>

		<div class="status-grid">
			<!-- System Health -->
			<div class="status-card" :class="health.status">
				<div class="status-icon">
					<span v-if="health.status === 'healthy'">💚</span>
					<span v-else-if="health.status === 'warning'">⚠️</span>
					<span v-else-if="health.status === 'critical'">🔴</span>
					<span v-else>❓</span>
				</div>
				<div class="status-content">
					<div class="status-label">System Health</div>
					<div class="status-value">{{ health.status.toUpperCase() }}</div>
					<div class="status-score">Score: {{ health.score }}/100</div>
				</div>
			</div>

			<!-- Audio Status -->
			<div class="status-card" :class="{ active: stats?.audio.is_capturing }">
				<div class="status-icon">🎧</div>
				<div class="status-content">
					<div class="status-label">Audio Capture</div>
					<div class="status-value">{{ stats?.audio.is_capturing ? 'ACTIVE' : 'INACTIVE' }}</div>
					<div class="status-detail">Gain: {{ (stats?.audio.gain || 1).toFixed(1) }}x</div>
				</div>
			</div>

			<!-- LED Status -->
			<div class="status-card" :class="{ active: stats?.led.is_running }">
				<div class="status-icon">💡</div>
				<div class="status-content">
					<div class="status-label">LED Output</div>
					<div class="status-value">{{ stats?.led.is_running ? 'RUNNING' : 'STOPPED' }}</div>
					<div class="status-detail">{{ stats?.led.controllers || 0 }} controllers</div>
				</div>
			</div>

			<!-- Effects Status -->
			<div class="status-card active">
				<div class="status-icon">🎇</div>
				<div class="status-content">
					<div class="status-label">Current Effect</div>
					<div class="status-value">{{ stats?.effects.current_effect || 'None' }}</div>
					<div class="status-detail" v-if="stats?.effects.transitioning">Transitioning...</div>
				</div>
			</div>

			<!-- Performance -->
			<div class="status-card">
				<div class="status-icon">⚡</div>
				<div class="status-content">
					<div class="status-label">Performance</div>
					<div class="status-value">{{ stats?.performance.fps || 0 }} FPS</div>
					<div class="status-detail">{{ (uptime / 1000 / 60).toFixed(0) }}m uptime</div>
				</div>
			</div>

			<!-- Connection Quality -->
			<div class="status-card" :class="connectionClass">
				<div class="status-icon">📡</div>
				<div class="status-content">
					<div class="status-label">Connection</div>
					<div class="status-value">{{ isOnline ? 'ONLINE' : 'OFFLINE' }}</div>
					<div class="status-detail">{{ connectionQuality }}% quality</div>
				</div>
			</div>
		</div>

		<!-- Issues Section -->
		<div v-if="health.issues.length > 0" class="issues-section">
			<h3>⚠️ Active Issues</h3>
			<div class="issues-list">
				<div v-for="issue in health.issues" :key="issue" class="issue-item">
					{{ issue }}
				</div>
			</div>
		</div>

		<!-- Quick Actions -->
		<div class="quick-actions">
			<button class="action-btn primary" :disabled="loading" @click="$emit('health-check')">
				🩺 Health Check
			</button>
			<button class="action-btn secondary" :disabled="loading" @click="$emit('refresh-stats')">🔄 Refresh</button>
		</div>
	</div>
</template>

<script setup lang="ts">
	import { computed } from 'vue';
	import type { SystemHealth, SystemStats } from '../types';

	interface Props {
		stats: SystemStats | null;
		health: SystemHealth;
		isOnline: boolean;
		loading: boolean;
		connectionQuality: number;
		uptime: number;
	}

	interface Emits {
		(e: 'health-check'): void;
		(e: 'refresh-stats'): void;
	}

	const props = defineProps<Props>();
	defineEmits<Emits>();

	// Computed property pour éviter l'erreur
	const connectionClass = computed(() => {
		if (!props.isOnline) return 'critical';
		if (props.connectionQuality >= 80) return 'healthy';
		if (props.connectionQuality >= 50) return 'warning';
		return 'critical';
	});
</script>

<style scoped>
	.panel {
		background: #161b22;
		border: 1px solid #30363d;
		border-radius: 12px;
		padding: 1.5rem;
		margin-bottom: 1.5rem;
	}

	.panel:hover {
		border-color: #484f58;
	}

	.panel-header {
		margin-bottom: 1.5rem;
	}

	.panel-header h2 {
		margin: 0 0 0.5rem 0;
		font-size: 1.25rem;
		font-weight: 600;
		color: #f0f6fc;
	}

	.panel-subtitle {
		color: #8b949e;
		font-size: 0.875rem;
	}

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
		border: 1px solid #30363d;
		border-radius: 8px;
		background: #0d1117;
		transition: all 0.2s ease;
	}

	.status-card.active {
		background: #0d4929;
		border-color: #2ea043;
	}

	.status-card.healthy {
		background: #0d4929;
		border-color: #2ea043;
	}

	.status-card.warning {
		background: #4d3800;
		border-color: #d29922;
	}

	.status-card.critical {
		background: #4d0d0d;
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
		color: #8b949e;
		text-transform: uppercase;
		font-weight: 600;
		margin-bottom: 0.25rem;
	}

	.status-value {
		font-size: 0.875rem;
		font-weight: 600;
		color: #f0f6fc;
		margin-bottom: 0.25rem;
	}

	.status-detail {
		font-size: 0.75rem;
		color: #8b949e;
		font-family: 'SF Mono', Consolas, monospace;
	}

	.issues-section {
		margin-bottom: 1.5rem;
	}

	.issues-section h3 {
		margin: 0 0 1rem 0;
		font-size: 1rem;
		color: #d29922;
	}

	.issues-list {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.issue-item {
		padding: 0.5rem 1rem;
		background: #4d3800;
		border: 1px solid #d29922;
		border-radius: 6px;
		font-size: 0.875rem;
		color: #f0f6fc;
	}

	.quick-actions {
		display: flex;
		gap: 0.75rem;
	}

	.action-btn {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.75rem 1rem;
		border: 1px solid #30363d;
		border-radius: 6px;
		font-size: 0.875rem;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.action-btn.primary {
		background: #2ea043;
		border-color: #2ea043;
		color: white;
	}

	.action-btn.primary:hover:not(:disabled) {
		background: #2c974b;
	}

	.action-btn.secondary {
		background: #21262d;
		color: #f0f6fc;
	}

	.action-btn.secondary:hover:not(:disabled) {
		background: #30363d;
	}

	.action-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	@media (max-width: 768px) {
		.status-grid {
			grid-template-columns: 1fr;
		}

		.quick-actions {
			flex-direction: column;
		}
	}
</style>
