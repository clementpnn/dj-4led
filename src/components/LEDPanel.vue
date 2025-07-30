<template>
	<div class="led-panel">
		<div class="panel-header">
			<h2>LED Output</h2>
		</div>

		<!-- Controls -->
		<div class="controls-section">
			<div class="control-group">
				<label class="control-label">Brightness {{ Math.round(led.brightness * 100) }}%</label>
				<input
					type="range"
					class="brightness-slider"
					:value="led.brightness"
					min="0"
					max="1"
					step="0.05"
					:disabled="led.loading"
					@input="handleBrightnessChange"
				/>
			</div>

			<div class="control-group">
				<label class="control-label">Mode</label>
				<select class="mode-select" :value="led.currentMode" :disabled="led.loading" @change="handleModeChange">
					<option value="simulator">Simulator</option>
					<option value="production">Production</option>
				</select>
			</div>

			<button
				class="control-btn"
				:class="{ active: led.isRunning }"
				:disabled="led.loading"
				@click="handleOutputToggle"
			>
				{{ led.isRunning ? 'Stop' : 'Start' }}
			</button>
		</div>

		<!-- Frame Display -->
		<div class="frame-section">
			<div class="frame-container" :class="{ active: led.isRunning }">
				<div v-if="frames.hasCurrentFrame" class="frame-display">
					<img :src="currentFrameImageUrl" alt="LED Frame" class="frame-image" />
					<div class="frame-overlay">
						<span class="frame-info">{{ led.matrixSize }}</span>
						<span class="frame-fps">{{ frames.stats.fps }} FPS</span>
					</div>
				</div>
				<div v-else class="no-frame">
					<div class="no-frame-text">
						{{ led.isRunning ? 'Waiting for frames...' : 'LED output stopped' }}
					</div>
				</div>
			</div>
		</div>

		<!-- Metrics -->
		<div class="metrics-section">
			<div class="metric">
				<span class="metric-label">Frames</span>
				<span class="metric-value">{{ frames.frameCount }}</span>
			</div>
			<div class="metric">
				<span class="metric-label">Success</span>
				<span class="metric-value">{{ frames.successRate.value.toFixed(1) }}%</span>
			</div>
			<div class="metric">
				<span class="metric-label">Controllers</span>
				<span class="metric-value">{{ led.controllerCount }}</span>
			</div>
		</div>

		<!-- Status -->
		<div class="status-section">
			<div class="status-item">
				<span class="status-label">Output</span>
				<span class="status-value" :class="{ active: led.isRunning }">
					{{ led.isRunning ? 'RUNNING' : 'STOPPED' }}
				</span>
			</div>
			<div class="status-item">
				<span class="status-label">Health</span>
				<span class="status-value" :class="{ active: led.isHealthy }">
					{{ led.isHealthy ? 'GOOD' : 'ISSUES' }}
				</span>
			</div>
		</div>

		<!-- Frame Controls -->
		<div class="frame-controls-section">
			<div class="section-header">
				<h3>Frame Controls</h3>
				<div class="health-indicator" :class="frames.healthStatus">
					{{ frames.healthStatus.value.toUpperCase() }}
				</div>
			</div>
			<div class="frame-controls">
				<button class="control-btn secondary" :disabled="frames.loading" @click="handleRefreshFrame">
					{{ frames.loading ? 'Refreshing...' : 'Refresh Frame' }}
				</button>
				<button class="control-btn secondary" :disabled="!frames.hasCurrentFrame" @click="handleDownloadFrame">
					Download Frame
				</button>
				<button class="control-btn secondary" @click="frames.toggleAutoRefresh">
					Auto: {{ frames.autoRefresh ? 'ON' : 'OFF' }}
				</button>
			</div>
		</div>

		<!-- Test Controls -->
		<div class="test-section">
			<div class="section-header">
				<h3>Test Controls</h3>
			</div>
			<div class="test-controls">
				<button class="test-btn" :disabled="led.loading" @click="handleTestConnectivity">
					Test Connectivity
				</button>
				<button class="test-btn" :disabled="led.loading" @click="handleClearDisplay">Clear Display</button>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
	import { computed } from 'vue';
	import { useFrames } from '../composables/useFrames';
	import { useLED } from '../composables/useLED';

	// Composables
	const led = useLED();
	const frames = useFrames();

	// Computed properties
	const currentFrameImageUrl = computed(() => {
		if (frames.currentFrame) {
			return frames.frameToImageUrl(frames.currentFrame);
		}
		return '';
	});

	// LED Handlers
	const handleBrightnessChange = async (event: Event): Promise<void> => {
		const target = event.target as HTMLInputElement;
		if (target) {
			try {
				await led.setBrightness(Number(target.value));
			} catch (error) {
				console.error('Failed to change brightness:', error);
			}
		}
	};

	const handleModeChange = async (event: Event): Promise<void> => {
		const target = event.target as HTMLSelectElement;
		if (target) {
			try {
				// Stop current output if running
				if (led.isRunning) {
					await led.stopOutput();
				}
				// Start with new mode
				await led.startOutput(target.value as 'simulator' | 'production');
			} catch (error) {
				console.error('Failed to change mode:', error);
			}
		}
	};

	const handleOutputToggle = async (): Promise<void> => {
		try {
			if (led.isRunning) {
				await led.stopOutput();
			} else {
				await led.startOutput(led.currentMode);
			}
		} catch (error) {
			console.error('Failed to toggle output:', error);
		}
	};

	const handleTestConnectivity = async (): Promise<void> => {
		try {
			await led.testConnectivity();
		} catch (error) {
			console.error('Failed to test connectivity:', error);
		}
	};

	const handleClearDisplay = async (): Promise<void> => {
		try {
			await led.clearDisplay();
		} catch (error) {
			console.error('Failed to clear display:', error);
		}
	};

	// Frame Handlers
	const handleRefreshFrame = async (): Promise<void> => {
		try {
			await frames.refreshFrame();
		} catch (error) {
			console.error('Failed to refresh frame:', error);
		}
	};

	const handleDownloadFrame = async (): Promise<void> => {
		try {
			await frames.downloadFrame();
		} catch (error) {
			console.error('Failed to download frame:', error);
		}
	};
</script>

<style scoped>
	.led-panel {
		background: #0d1117;
		border: 1px solid #30363d;
		border-radius: 8px;
		padding: 1.5rem;
		color: #c9d1d9;
	}

	.panel-header {
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

	.controls-section {
		display: flex;
		align-items: end;
		gap: 1rem;
		margin-bottom: 1.5rem;
	}

	.control-group {
		display: flex;
		flex-direction: column;
		gap: 0.375rem;
		flex: 1;
	}

	.control-label {
		font-size: 0.75rem;
		font-weight: 500;
		color: #7d8590;
		text-transform: uppercase;
		letter-spacing: 0.025em;
	}

	.brightness-slider {
		width: 100%;
		height: 4px;
		background: #21262d;
		border-radius: 2px;
		outline: none;
		appearance: none;
		-webkit-appearance: none;
	}

	.brightness-slider::-webkit-slider-thumb {
		appearance: none;
		-webkit-appearance: none;
		width: 16px;
		height: 16px;
		background: #c9d1d9;
		border-radius: 50%;
		cursor: pointer;
	}

	.brightness-slider::-moz-range-thumb {
		width: 16px;
		height: 16px;
		background: #c9d1d9;
		border-radius: 50%;
		border: none;
		cursor: pointer;
	}

	.mode-select {
		padding: 0.5rem 0.75rem;
		background: #21262d;
		border: 1px solid #30363d;
		border-radius: 4px;
		color: #c9d1d9;
		font-size: 0.875rem;
	}

	.mode-select:focus {
		outline: none;
		border-color: #c9d1d9;
	}

	.mode-select:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.control-btn {
		padding: 0.75rem 1.5rem;
		background: #21262d;
		border: 1px solid #30363d;
		border-radius: 4px;
		color: #c9d1d9;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s ease;
		min-width: 80px;
	}

	.control-btn:hover:not(:disabled) {
		background: #30363d;
	}

	.control-btn.active {
		background: #c9d1d9;
		color: #0d1117;
		border-color: #c9d1d9;
	}

	.control-btn.secondary {
		background: #161b22;
		color: #7d8590;
		border-color: #21262d;
		min-width: auto;
		padding: 0.5rem 1rem;
		font-size: 0.875rem;
	}

	.control-btn.secondary:hover:not(:disabled) {
		background: #21262d;
		color: #c9d1d9;
	}

	.control-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.frame-section {
		margin-bottom: 1.5rem;
	}

	.frame-container {
		height: 200px;
		background: #161b22;
		border: 1px solid #21262d;
		border-radius: 4px;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: border-color 0.3s ease;
		position: relative;
	}

	.frame-container.active {
		border-color: #c9d1d9;
	}

	.frame-display {
		position: relative;
		max-width: 100%;
		max-height: 100%;
	}

	.frame-image {
		max-width: 100%;
		max-height: 180px;
		border-radius: 4px;
		image-rendering: pixelated;
	}

	.frame-overlay {
		position: absolute;
		top: 0.5rem;
		right: 0.5rem;
		background: rgba(0, 0, 0, 0.8);
		padding: 0.25rem 0.5rem;
		border-radius: 3px;
		font-size: 0.75rem;
		color: #c9d1d9;
		font-family: monospace;
		display: flex;
		gap: 0.5rem;
	}

	.no-frame {
		text-align: center;
		color: #7d8590;
		font-size: 0.875rem;
	}

	.metrics-section {
		display: flex;
		gap: 1rem;
		margin-bottom: 1rem;
	}

	.metric {
		flex: 1;
		text-align: center;
		padding: 0.75rem;
		background: #161b22;
		border: 1px solid #21262d;
		border-radius: 4px;
	}

	.metric-label {
		display: block;
		font-size: 0.75rem;
		color: #7d8590;
		text-transform: uppercase;
		margin-bottom: 0.25rem;
	}

	.metric-value {
		display: block;
		font-size: 1rem;
		font-weight: 600;
		color: #c9d1d9;
		font-family: monospace;
	}

	.status-section {
		display: flex;
		gap: 1rem;
		padding: 1rem;
		background: #161b22;
		border: 1px solid #21262d;
		border-radius: 4px;
		margin-bottom: 1.5rem;
	}

	.status-item {
		display: flex;
		justify-content: space-between;
		flex: 1;
	}

	.status-label {
		font-size: 0.75rem;
		color: #7d8590;
		font-weight: 500;
		text-transform: uppercase;
	}

	.status-value {
		font-size: 0.75rem;
		color: #c9d1d9;
		font-family: monospace;
		font-weight: 600;
	}

	.status-value.active {
		color: #c9d1d9;
	}

	/* Frame Controls Section */
	.frame-controls-section {
		margin-bottom: 1.5rem;
	}

	.section-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1rem;
	}

	.section-header h3 {
		margin: 0;
		font-size: 1rem;
		font-weight: 600;
		color: #c9d1d9;
	}

	.health-indicator {
		padding: 0.25rem 0.5rem;
		border-radius: 3px;
		font-size: 0.625rem;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.025em;
	}

	.health-indicator.healthy {
		background: rgba(63, 185, 80, 0.1);
		color: #3fb950;
		border: 1px solid rgba(63, 185, 80, 0.3);
	}

	.health-indicator.warning {
		background: rgba(210, 153, 34, 0.1);
		color: #d29922;
		border: 1px solid rgba(210, 153, 34, 0.3);
	}

	.health-indicator.critical {
		background: rgba(248, 81, 73, 0.1);
		color: #f85149;
		border: 1px solid rgba(248, 81, 73, 0.3);
	}

	.frame-controls {
		display: flex;
		gap: 0.75rem;
	}

	.test-section {
		margin-bottom: 0;
	}

	.test-controls {
		display: flex;
		gap: 0.75rem;
	}

	.test-btn {
		padding: 0.5rem 1rem;
		background: #21262d;
		border: 1px solid #30363d;
		border-radius: 4px;
		color: #c9d1d9;
		font-size: 0.875rem;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.test-btn:hover:not(:disabled) {
		background: #30363d;
		border-color: #c9d1d9;
	}

	.test-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	@media (max-width: 768px) {
		.controls-section {
			flex-direction: column;
			align-items: stretch;
		}

		.control-btn {
			margin-top: 0.5rem;
		}

		.metrics-section {
			flex-direction: column;
		}

		.status-section {
			flex-direction: column;
			gap: 0.5rem;
		}

		.frame-container {
			height: 150px;
		}

		.frame-controls,
		.test-controls {
			flex-direction: column;
		}
	}
</style>
