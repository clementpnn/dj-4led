<template>
	<div class="led-panel">
		<div class="panel-header">
			<h2>LED Output</h2>
		</div>

		<!-- Controls -->
		<div class="controls-section">
			<div class="control-group">
				<label class="control-label">Brightness {{ Math.round(brightness * 100) }}%</label>
				<input
					type="range"
					class="brightness-slider"
					:value="brightness"
					min="0"
					max="1"
					step="0.05"
					:disabled="loading"
					@input="handleBrightnessChange"
				/>
			</div>

			<div class="control-group">
				<label class="control-label">Mode</label>
				<select class="mode-select" :value="currentMode" :disabled="loading" @change="handleModeChange">
					<option value="simulator">Simulator</option>
					<option value="production">Production</option>
				</select>
			</div>

			<button
				class="control-btn"
				:class="{ active: isRunning }"
				:disabled="loading"
				@click="$emit('output-toggle')"
			>
				{{ isRunning ? 'Stop' : 'Start' }}
			</button>
		</div>

		<!-- Frame Display -->
		<div class="frame-section">
			<div class="frame-container" :class="{ active: isRunning }">
				<div v-if="frameImageUrl" class="frame-display">
					<img :src="frameImageUrl" alt="LED Frame" class="frame-image" />
					<div class="frame-overlay">
						<span class="frame-info">{{ currentFrame?.width }}x{{ currentFrame?.height }}</span>
						<span class="frame-fps">{{ frameRate }} FPS</span>
					</div>
				</div>
				<div v-else class="no-frame">
					<div class="no-frame-text">
						{{ isRunning ? 'Waiting for frames...' : 'LED output stopped' }}
					</div>
				</div>
			</div>
		</div>

		<!-- Metrics -->
		<div class="metrics-section">
			<div class="metric">
				<span class="metric-label">Frames</span>
				<span class="metric-value">{{ frameCount }}</span>
			</div>
			<div class="metric">
				<span class="metric-label">Success</span>
				<span class="metric-value">{{ metrics.successRate.toFixed(1) }}%</span>
			</div>
			<div class="metric">
				<span class="metric-label">Controllers</span>
				<span class="metric-value">{{ controllerCount }}</span>
			</div>
		</div>

		<!-- Status -->
		<div class="status-section">
			<div class="status-item">
				<span class="status-label">Output</span>
				<span class="status-value" :class="{ active: isRunning }">
					{{ isRunning ? 'RUNNING' : 'STOPPED' }}
				</span>
			</div>
			<div class="status-item">
				<span class="status-label">Health</span>
				<span class="status-value" :class="{ active: isHealthy }">
					{{ isHealthy ? 'GOOD' : 'ISSUES' }}
				</span>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
	import type { FrameData, FrameMetrics, FrameStats, LEDController } from '../types';

	interface Props {
		isRunning: boolean;
		brightness: number;
		currentMode: string;
		loading: boolean;
		currentFrame: FrameData | null;
		frameImageUrl: string;
		frameRate: number;
		frameCount: number;
		stats: FrameStats | null;
		metrics: FrameMetrics;
		controllerCount: number;
		connectedControllers: LEDController[];
		frameSize: number;
		matrixSize: string;
		isHealthy: boolean;
		hasCurrentFrame: boolean;
	}

	interface Emits {
		(e: 'output-toggle'): void;
		(e: 'brightness-change', brightness: number): void;
		(e: 'mode-change', mode: string): void;
		(e: 'test-pattern'): void;
	}

	defineProps<Props>();
	const emit = defineEmits<Emits>();

	const handleBrightnessChange = (event: Event): void => {
		const target = event.target as HTMLInputElement;
		if (target) {
			emit('brightness-change', Number(target.value));
		}
	};

	const handleModeChange = (event: Event): void => {
		const target = event.target as HTMLSelectElement;
		if (target) {
			emit('mode-change', target.value);
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
	}
</style>
