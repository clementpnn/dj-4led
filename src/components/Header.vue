<template>
	<div class="header">
		<div class="header-left">
			<div class="brand">
				<span class="logo">🎛️</span>
				<h1>DJ4LED</h1>
			</div>
			<div class="status-info">
				<div class="connection-status" :class="{ connected: isConnected }">
					<div class="status-dot"></div>
					<span class="status-text">{{ isConnected ? 'Connected' : 'Offline' }}</span>
				</div>
				<div v-if="isConnected && fps > 0" class="fps-counter">{{ fps }} FPS</div>
			</div>
		</div>

		<div class="header-actions">
			<!-- Stream Toggle -->
			<button
				v-if="isConnected"
				class="action-btn stream-btn"
				:class="{ active: isStreaming, loading: streamLoading }"
				:disabled="streamLoading"
				@click="$emit('stream-toggle')"
			>
				<span class="btn-icon">{{ isStreaming ? '⏹️' : '▶️' }}</span>
				<span class="btn-text">{{ isStreaming ? 'Stop Stream' : 'Start Stream' }}</span>
			</button>

			<!-- Quick Actions -->
			<button class="action-btn primary" :disabled="loading" @click="handleConnect">
				<span class="btn-icon">{{ isConnected ? '🔌' : '🔗' }}</span>
				<span class="btn-text">{{ isConnected ? 'Disconnect' : 'Connect' }}</span>
			</button>

			<button class="action-btn secondary" :disabled="!isConnected || loading" @click="handlePing">
				<span class="btn-icon">📡</span>
				<span class="btn-text">Health Check</span>
				<span v-if="pingMs > 0" class="ping-badge">{{ pingMs }}ms</span>
			</button>

			<!-- Health Indicator -->
			<div v-if="isStreaming" class="health-status" :class="{ healthy: isStreamHealthy }">
				<div class="health-dot"></div>
				<span class="health-label">{{ isStreamHealthy ? 'Healthy' : 'Issues' }}</span>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
	interface Props {
		isConnected: boolean;
		fps: number;
		isStreaming: boolean;
		streamLoading: boolean;
		isStreamHealthy: boolean;
		loading: boolean;
		pingMs: number;
	}

	interface Emits {
		(e: 'connect'): void;
		(e: 'disconnect'): void;
		(e: 'ping'): void;
		(e: 'stream-toggle'): void;
	}

	const props = withDefaults(defineProps<Props>(), {
		isConnected: false,
		fps: 0,
		isStreaming: false,
		streamLoading: false,
		isStreamHealthy: true,
		loading: false,
		pingMs: 0,
	});

	const emit = defineEmits<Emits>();

	const handleConnect = (): void => {
		if (props.isConnected) {
			emit('disconnect');
		} else {
			emit('connect');
		}
	};

	const handlePing = (): void => {
		emit('ping');
	};
</script>

<style scoped>
	.header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 1rem 1.5rem;
		background: #0d1117;
		border-bottom: 1px solid #21262d;
		color: #c9d1d9;
		width: 100%;
		box-sizing: border-box;
	}

	/* Brand Section */
	.header-left {
		display: flex;
		align-items: center;
		gap: 2rem;
	}

	.brand {
		display: flex;
		align-items: center;
		gap: 0.75rem;
	}

	.logo {
		font-size: 1.5rem;
	}

	.brand h1 {
		margin: 0;
		font-size: 1.25rem;
		font-weight: 600;
		color: #c9d1d9;
		letter-spacing: -0.025em;
	}

	/* Status Info */
	.status-info {
		display: flex;
		align-items: center;
		gap: 1rem;
	}

	.connection-status {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.375rem 0.75rem;
		background: #161b22;
		border: 1px solid #21262d;
		border-radius: 4px;
		font-size: 0.75rem;
		font-weight: 500;
	}

	.connection-status.connected {
		border-color: #3fb950;
		background: rgba(63, 185, 80, 0.05);
	}

	.connection-status.connected .status-text {
		color: #3fb950;
	}

	.status-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: #f85149;
	}

	.connection-status.connected .status-dot {
		background: #3fb950;
		animation: pulse 2s infinite;
	}

	.status-text {
		color: #f85149;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.025em;
	}

	.fps-counter {
		padding: 0.375rem 0.75rem;
		background: #161b22;
		border: 1px solid #21262d;
		border-radius: 4px;
		font-size: 0.75rem;
		font-weight: 600;
		color: #79c0ff;
		font-family: monospace;
	}

	/* Actions */
	.header-actions {
		display: flex;
		align-items: center;
		gap: 0.75rem;
	}

	.action-btn {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.75rem 1rem;
		background: #161b22;
		border: 1px solid #21262d;
		border-radius: 4px;
		color: #c9d1d9;
		font-size: 0.875rem;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s ease;
		white-space: nowrap;
	}

	.action-btn:hover:not(:disabled) {
		background: #21262d;
		border-color: #30363d;
	}

	.action-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.action-btn.primary {
		border-color: #3fb950;
		color: #3fb950;
	}

	.action-btn.primary:hover:not(:disabled) {
		background: rgba(63, 185, 80, 0.1);
		border-color: #46c557;
	}

	.action-btn.secondary {
		border-color: #79c0ff;
		color: #79c0ff;
	}

	.action-btn.secondary:hover:not(:disabled) {
		background: rgba(121, 192, 255, 0.1);
		border-color: #8cc8ff;
	}

	.action-btn.stream-btn {
		border-color: #f85149;
		color: #f85149;
	}

	.action-btn.stream-btn:hover:not(:disabled) {
		background: rgba(248, 81, 73, 0.1);
		border-color: #ff6b62;
	}

	.action-btn.stream-btn.active {
		background: rgba(248, 81, 73, 0.1);
		border-color: #f85149;
	}

	.action-btn.stream-btn.loading {
		opacity: 0.7;
		cursor: wait;
	}

	.btn-icon {
		font-size: 1rem;
		flex-shrink: 0;
	}

	.btn-text {
		font-weight: 500;
	}

	.ping-badge {
		padding: 0.125rem 0.375rem;
		background: #21262d;
		border-radius: 8px;
		font-size: 0.625rem;
		font-weight: 600;
		font-family: monospace;
	}

	/* Health Status */
	.health-status {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.375rem 0.75rem;
		background: #161b22;
		border: 1px solid #21262d;
		border-radius: 4px;
		font-size: 0.75rem;
		font-weight: 500;
	}

	.health-status.healthy {
		border-color: #3fb950;
		background: rgba(63, 185, 80, 0.05);
	}

	.health-status.healthy .health-label {
		color: #3fb950;
	}

	.health-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: #f85149;
	}

	.health-status.healthy .health-dot {
		background: #3fb950;
	}

	.health-label {
		color: #f85149;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.025em;
	}

	/* Animations */
	@keyframes pulse {
		0%,
		100% {
			opacity: 1;
		}
		50% {
			opacity: 0.5;
		}
	}

	/* Responsive */
	@media (max-width: 1024px) {
		.header {
			padding: 1rem;
		}

		.header-left {
			gap: 1.5rem;
		}

		.header-actions {
			gap: 0.5rem;
		}

		.action-btn {
			padding: 0.625rem 0.875rem;
			font-size: 0.8rem;
		}
	}

	@media (max-width: 768px) {
		.header {
			flex-direction: column;
			gap: 1rem;
			padding: 1rem;
		}

		.header-left {
			width: 100%;
			justify-content: space-between;
		}

		.header-actions {
			width: 100%;
			justify-content: center;
			flex-wrap: wrap;
		}

		.action-btn {
			flex: 1;
			min-width: 120px;
			justify-content: center;
		}
	}

	@media (max-width: 480px) {
		.brand h1 {
			font-size: 1.125rem;
		}

		.status-info {
			flex-direction: column;
			gap: 0.5rem;
			align-items: flex-end;
		}

		.action-btn {
			padding: 0.75rem;
			font-size: 0.875rem;
		}

		.btn-text {
			display: none;
		}

		.health-status {
			order: -1;
		}
	}
</style>
