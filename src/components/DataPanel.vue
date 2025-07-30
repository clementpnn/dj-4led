<template>
	<div class="data-panel">
		<!-- Header -->
		<div class="panel-header">
			<h2>System Data</h2>
			<div class="status-indicator" :class="getStatusClass()">
				<div class="status-dot"></div>
				<span>{{ getStatusText() }}</span>
			</div>
		</div>

		<!-- Real-time Metrics -->
		<div class="metrics-section">
			<div class="metric-group">
				<div class="metric-item">
					<span class="metric-label">FPS</span>
					<span class="metric-value">{{ frames.stats.fps || 0 }}</span>
				</div>
				<div class="metric-item">
					<span class="metric-label">Stream</span>
					<span class="metric-value" :class="{ active: isStreaming }">
						{{ isStreaming ? 'LIVE' : 'IDLE' }}
					</span>
				</div>
				<div class="metric-item">
					<span class="metric-label">Uptime</span>
					<span class="metric-value">{{ system.systemUptime }}</span>
				</div>
			</div>
		</div>

		<!-- Audio Data -->
		<div class="data-section">
			<div class="section-header">
				<span class="section-title">Audio</span>
				<div class="section-indicator" :class="{ active: audio.state.isCapturing }"></div>
			</div>
			<div class="data-grid">
				<div class="data-item">
					<span class="data-label">Status</span>
					<span class="data-value">{{ audio.state.isCapturing ? 'Capturing' : 'Stopped' }}</span>
				</div>
				<div class="data-item">
					<span class="data-label">Gain</span>
					<span class="data-value">{{ audio.state.currentGain?.toFixed(1) || '1.0' }}x</span>
				</div>
				<div class="data-item">
					<span class="data-label">Devices</span>
					<span class="data-value">{{ audio.state.devices.length || 0 }}</span>
				</div>
				<div class="data-item">
					<span class="data-label">Spectrum</span>
					<span class="data-value">{{ audio.state.spectrum?.length || 0 }} bands</span>
				</div>
				<div class="data-item">
					<span class="data-label">Peak</span>
					<span class="data-value">{{ Math.round((audio.spectrumPeak || 0) * 100) }}%</span>
				</div>
				<div class="data-item">
					<span class="data-label">RMS</span>
					<span class="data-value">{{ Math.round((audio.spectrumRMS || 0) * 100) }}%</span>
				</div>
			</div>
		</div>

		<!-- LED Data -->
		<div class="data-section">
			<div class="section-header">
				<span class="section-title">LED</span>
				<div class="section-indicator" :class="{ active: led.isRunning }"></div>
			</div>
			<div class="data-grid">
				<div class="data-item">
					<span class="data-label">Status</span>
					<span class="data-value">{{ led.isRunning ? 'Running' : 'Stopped' }}</span>
				</div>
				<div class="data-item">
					<span class="data-label">Mode</span>
					<span class="data-value">{{ led.currentMode }}</span>
				</div>
				<div class="data-item">
					<span class="data-label">Brightness</span>
					<span class="data-value">{{ Math.round(led.brightness * 100) }}%</span>
				</div>
				<div class="data-item">
					<span class="data-label">Controllers</span>
					<span class="data-value">{{ led.controllerCount }}</span>
				</div>
				<div class="data-item">
					<span class="data-label">Frame Rate</span>
					<span class="data-value">{{ frames.stats.fps || 0 }} fps</span>
				</div>
				<div class="data-item">
					<span class="data-label">Frame Size</span>
					<span class="data-value">{{ led.matrixSize }}</span>
				</div>
			</div>
		</div>

		<!-- Effects Data -->
		<div class="data-section">
			<div class="section-header">
				<span class="section-title">Effects</span>
				<div class="section-indicator active"></div>
			</div>
			<div class="data-grid">
				<div class="data-item">
					<span class="data-label">Current</span>
					<span class="data-value">{{ effects.currentEffectName }}</span>
				</div>
				<div class="data-item">
					<span class="data-label">Available</span>
					<span class="data-value">{{ effects.availableEffects.length }}</span>
				</div>
				<div class="data-item">
					<span class="data-label">Color Mode</span>
					<span class="data-value">{{ colors.currentModeInfo?.label || colors.currentMode }}</span>
				</div>
				<div class="data-item">
					<span class="data-label">Transitioning</span>
					<span class="data-value">{{ effects.isTransitioning ? 'Yes' : 'No' }}</span>
				</div>
			</div>
		</div>

		<!-- System Data -->
		<div class="data-section">
			<div class="section-header">
				<span class="section-title">System</span>
				<div class="section-indicator" :class="{ active: system.isHealthy }"></div>
			</div>
			<div class="data-grid">
				<div class="data-item">
					<span class="data-label">Connection</span>
					<span class="data-value">{{ connectionStatus }}</span>
				</div>
				<div class="data-item">
					<span class="data-label">Health</span>
					<span class="data-value">{{ system.health.status }}</span>
				</div>
				<div class="data-item">
					<span class="data-label">Score</span>
					<span class="data-value">{{ system.health.score }}/100</span>
				</div>
				<div class="data-item">
					<span class="data-label">Monitoring</span>
					<span class="data-value">{{ system.loading ? 'Active' : 'Inactive' }}</span>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
	import { computed } from 'vue';
	import { useAudio } from '../composables/useAudio';
	import { useColors } from '../composables/useColors';
	import { useEffects } from '../composables/useEffects';
	import { useFrames } from '../composables/useFrames';
	import { useLED } from '../composables/useLED';
	import { useSystem } from '../composables/useSystem';

	// Props pour les données de connexion qui peuvent venir d'ailleurs
	interface Props {
		isConnected?: boolean;
	}

	const props = withDefaults(defineProps<Props>(), {
		isConnected: true,
	});

	// Composables
	const audio = useAudio();
	const effects = useEffects();
	const colors = useColors();
	const led = useLED();
	const frames = useFrames();
	const system = useSystem();

	// Computed properties
	const isStreaming = computed(() => {
		return audio.state.isCapturing && led.isRunning;
	});

	const connectionStatus = computed(() => {
		return props.isConnected ? 'Online' : 'Offline';
	});

	// Status methods
	const getStatusClass = (): string => {
		if (!props.isConnected) return 'offline';
		if (!isStreaming.value) return 'ready';
		return 'live';
	};

	const getStatusText = (): string => {
		if (!props.isConnected) return 'OFFLINE';
		if (!isStreaming.value) return 'READY';
		return 'LIVE';
	};
</script>

<style scoped>
	.data-panel {
		background: #0d1117;
		border: 1px solid #30363d;
		border-radius: 8px;
		padding: 1.5rem;
		color: #c9d1d9;
		height: fit-content;
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

	.status-indicator {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.375rem 0.75rem;
		border-radius: 4px;
		font-size: 0.75rem;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.025em;
	}

	.status-indicator.offline {
		background: #161b22;
		color: #f85149;
		border: 1px solid #21262d;
	}

	.status-indicator.ready {
		background: #161b22;
		color: #79c0ff;
		border: 1px solid #21262d;
	}

	.status-indicator.live {
		background: #161b22;
		color: #3fb950;
		border: 1px solid #21262d;
	}

	.status-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: currentColor;
		animation: pulse 2s infinite;
	}

	@keyframes pulse {
		0%,
		100% {
			opacity: 1;
		}
		50% {
			opacity: 0.5;
		}
	}

	/* Metrics Section */
	.metrics-section {
		margin-bottom: 1.5rem;
	}

	.metric-group {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 1rem;
	}

	.metric-item {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.5rem;
		padding: 1rem;
		background: #161b22;
		border: 1px solid #21262d;
		border-radius: 4px;
	}

	.metric-label {
		font-size: 0.75rem;
		color: #7d8590;
		font-weight: 500;
		text-transform: uppercase;
		letter-spacing: 0.025em;
	}

	.metric-value {
		font-size: 1.25rem;
		font-weight: 700;
		color: #c9d1d9;
		font-family: monospace;
	}

	.metric-value.active {
		color: #3fb950;
	}

	/* Data Sections */
	.data-section {
		margin-bottom: 1.5rem;
	}

	.data-section:last-child {
		margin-bottom: 0;
	}

	.section-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 0.75rem;
	}

	.section-title {
		font-size: 0.875rem;
		font-weight: 600;
		color: #c9d1d9;
	}

	.section-indicator {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: #21262d;
	}

	.section-indicator.active {
		background: #3fb950;
	}

	.data-grid {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: 0.75rem;
	}

	.data-item {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 0.75rem;
		background: #161b22;
		border: 1px solid #21262d;
		border-radius: 4px;
	}

	.data-label {
		font-size: 0.75rem;
		color: #7d8590;
		font-weight: 500;
	}

	.data-value {
		font-size: 0.75rem;
		color: #c9d1d9;
		font-weight: 600;
		font-family: monospace;
		text-align: right;
	}

	/* Responsive */
	@media (max-width: 768px) {
		.data-panel {
			padding: 1rem;
		}

		.metric-group {
			grid-template-columns: 1fr;
		}

		.data-grid {
			grid-template-columns: 1fr;
		}

		.panel-header {
			flex-direction: column;
			align-items: flex-start;
			gap: 0.75rem;
		}
	}
</style>
