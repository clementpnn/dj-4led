<template>
	<div class="header">
		<div class="header-left">
			<h1>DJ-4LED</h1>
			<div class="status">
				<span :class="isConnected ? 'connected' : 'disconnected'">
					{{ isConnected ? '● Connected' : '○ Disconnected' }}
				</span>
				<span v-if="isConnected && fps > 0" class="fps">{{ fps }} FPS</span>
			</div>
		</div>

		<!-- Quick Actions intégrées -->
		<div class="header-actions">
			<button class="action-btn primary" :disabled="loading" @click="handleConnect">
				<span class="btn-text">{{ isConnected ? 'Disconnect' : 'Connect' }}</span>
			</button>

			<button class="action-btn secondary" :disabled="!isConnected || loading" @click="handlePing">
				<span class="btn-text">Ping</span>
				<span v-if="pingMs > 0" class="ping-value">({{ pingMs }}ms)</span>
			</button>

			<!-- Stream Controls -->
			<button
				v-if="isConnected"
				class="action-btn stream-btn"
				:class="{ active: isStreaming }"
				:disabled="streamLoading"
				@click="$emit('stream-toggle')"
			>
				<span class="btn-text">{{ isStreaming ? 'Stop' : 'Start' }}</span>
			</button>

			<!-- Health Indicator -->
			<div v-if="isStreaming" class="health-indicator" :class="{ healthy: isStreamHealthy }">
				<div class="health-dot"></div>
				<span class="health-text">{{ isStreamHealthy ? 'Good' : 'Poor' }}</span>
			</div>
		</div>
	</div>
</template>

<script setup>
	const props = defineProps({
		isConnected: {
			type: Boolean,
			default: false,
		},
		fps: {
			type: Number,
			default: 0,
		},
		isStreaming: {
			type: Boolean,
			default: false,
		},
		streamLoading: {
			type: Boolean,
			default: false,
		},
		isStreamHealthy: {
			type: Boolean,
			default: true,
		},
		loading: {
			type: Boolean,
			default: false,
		},
		pingMs: {
			type: Number,
			default: 0,
		},
	});

	const emit = defineEmits(['connect', 'disconnect', 'ping', 'stream-toggle']);

	const handleConnect = () => {
		if (props.isConnected) {
			emit('disconnect');
		} else {
			emit('connect');
		}
	};

	const handlePing = () => {
		emit('ping');
	};
</script>

<style scoped>
	.header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 1rem 2rem;
		background: rgba(22, 27, 34, 0.95);
		backdrop-filter: blur(10px);
		border-bottom: 1px solid #333;
		position: sticky;
		top: 0;
		z-index: 100;
		box-shadow: 0 2px 10px rgba(0, 0, 0, 0.2);
		gap: 2rem;
	}

	.header-left {
		display: flex;
		align-items: center;
		gap: 2rem;
		flex-shrink: 0;
	}

	.header h1 {
		margin: 0;
		font-size: 1.5rem;
		font-weight: 700;
		display: flex;
		align-items: center;
		gap: 0.5rem;
		color: #fff;
	}

	.logo {
		font-size: 1.75rem;
		filter: drop-shadow(0 0 8px rgba(88, 166, 255, 0.4));
	}

	.status {
		display: flex;
		align-items: center;
		gap: 1rem;
		font-size: 0.875rem;
	}

	.connected {
		color: #4ade80;
		font-weight: 600;
		text-shadow: 0 0 8px rgba(74, 222, 128, 0.3);
	}

	.disconnected {
		color: #f87171;
		font-weight: 600;
		text-shadow: 0 0 8px rgba(248, 113, 113, 0.3);
	}

	.fps {
		background: linear-gradient(45deg, #2563eb, #3b82f6);
		color: white;
		padding: 0.375rem 0.75rem;
		border-radius: 20px;
		font-weight: 700;
		font-size: 0.75rem;
		box-shadow: 0 2px 8px rgba(37, 99, 235, 0.3);
		border: 1px solid rgba(59, 130, 246, 0.3);
	}

	.header-actions {
		display: flex;
		align-items: center;
		gap: 1rem;
		flex-wrap: wrap;
	}

	.action-btn {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.75rem 1.25rem;
		border: 1px solid #333;
		border-radius: 25px;
		font-size: 0.875rem;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.3s ease;
		position: relative;
		overflow: hidden;
		white-space: nowrap;
	}

	.action-btn.primary {
		background: linear-gradient(135deg, #2d5016, #3d6a1f);
		border-color: #4ade80;
		color: white;
		box-shadow: 0 0 15px rgba(74, 222, 128, 0.3);
	}

	.action-btn.primary:hover:not(:disabled) {
		background: linear-gradient(135deg, #3d6a1f, #4d7a2f);
		box-shadow: 0 4px 20px rgba(74, 222, 128, 0.4);
		transform: translateY(-1px);
	}

	.action-btn.secondary {
		background: linear-gradient(135deg, #1e3a8a, #2563eb);
		border-color: #3b82f6;
		color: white;
		box-shadow: 0 0 15px rgba(59, 130, 246, 0.3);
	}

	.action-btn.secondary:hover:not(:disabled) {
		background: linear-gradient(135deg, #2563eb, #3b82f6);
		box-shadow: 0 4px 20px rgba(59, 130, 246, 0.4);
		transform: translateY(-1px);
	}

	.action-btn.stream-btn {
		background: linear-gradient(135deg, #2a2a2a, #3a3a3a);
		color: white;
	}

	.action-btn.stream-btn:hover:not(:disabled) {
		background: linear-gradient(135deg, #3a3a3a, #4a4a4a);
		transform: translateY(-1px);
		box-shadow: 0 4px 15px rgba(0, 0, 0, 0.3);
		border-color: #555;
	}

	.action-btn.stream-btn.active {
		background: linear-gradient(135deg, #dc2626, #ef4444);
		border-color: #f87171;
		box-shadow: 0 0 15px rgba(220, 38, 38, 0.4);
	}

	.action-btn.stream-btn.active:hover:not(:disabled) {
		background: linear-gradient(135deg, #ef4444, #f87171);
		box-shadow: 0 4px 20px rgba(220, 38, 38, 0.5);
	}

	.action-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
		transform: none;
		box-shadow: none;
	}

	.btn-icon {
		font-size: 1rem;
		flex-shrink: 0;
	}

	.btn-text {
		font-weight: 700;
		flex-shrink: 0;
	}

	.ping-value {
		font-size: 0.75rem;
		opacity: 0.9;
		font-weight: 500;
	}

	.health-indicator {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.5rem 1rem;
		background: rgba(30, 30, 30, 0.8);
		border: 1px solid #333;
		border-radius: 20px;
		font-size: 0.75rem;
		font-weight: 600;
		backdrop-filter: blur(4px);
	}

	.health-indicator.healthy {
		border-color: #4ade80;
		background: rgba(74, 222, 128, 0.1);
	}

	.health-indicator.healthy .health-text {
		color: #4ade80;
	}

	.health-indicator.healthy .health-dot {
		background: #4ade80;
		box-shadow: 0 0 8px rgba(74, 222, 128, 0.6);
	}

	.health-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: #f87171;
		box-shadow: 0 0 8px rgba(248, 113, 113, 0.6);
		animation: pulse 2s infinite;
	}

	.health-text {
		color: #f87171;
		font-weight: 700;
	}

	@keyframes pulse {
		0%,
		100% {
			opacity: 1;
			transform: scale(1);
		}
		50% {
			opacity: 0.7;
			transform: scale(1.1);
		}
	}

	/* Animation pour les boutons */
	.action-btn::before {
		content: '';
		position: absolute;
		top: 0;
		left: -100%;
		width: 100%;
		height: 100%;
		background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.1), transparent);
		transition: left 0.5s;
	}

	.action-btn:hover:not(:disabled)::before {
		left: 100%;
	}

	@media (max-width: 1200px) {
		.header {
			padding: 1rem 1.5rem;
			gap: 1.5rem;
		}

		.header-left {
			gap: 1.5rem;
		}

		.header-actions {
			gap: 0.75rem;
		}

		.action-btn {
			padding: 0.625rem 1rem;
			font-size: 0.8rem;
		}
	}

	@media (max-width: 1024px) {
		.header {
			flex-direction: column;
			gap: 1.5rem;
			padding: 1rem;
			align-items: stretch;
		}

		.header-left {
			justify-content: space-between;
			align-items: center;
			gap: 1rem;
		}

		.header h1 {
			font-size: 1.25rem;
		}

		.header-actions {
			justify-content: center;
			flex-wrap: wrap;
			gap: 1rem;
		}

		.action-btn {
			flex: 1;
			justify-content: center;
			min-width: 120px;
			max-width: 200px;
		}
	}

	@media (max-width: 768px) {
		.header-left {
			flex-direction: column;
			gap: 0.75rem;
			align-items: center;
		}

		.status {
			gap: 0.75rem;
			font-size: 0.8rem;
		}

		.header-actions {
			flex-direction: column;
			gap: 0.75rem;
		}

		.action-btn {
			width: 100%;
			max-width: none;
		}

		.health-indicator {
			align-self: center;
			padding: 0.375rem 0.75rem;
		}
	}

	@media (max-width: 480px) {
		.header {
			padding: 0.75rem;
		}

		.action-btn {
			padding: 0.75rem 1rem;
			font-size: 0.875rem;
		}

		.ping-value {
			display: none;
		}
	}

	/* Focus styles pour l'accessibilité */
	.action-btn:focus {
		outline: 2px solid #58a6ff;
		outline-offset: 2px;
	}
</style>
