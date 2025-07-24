<template>
	<div class="audio-panel">
		<div class="panel-header">
			<h2>Audio Spectrum</h2>
		</div>

		<!-- Controls -->
		<div class="controls-section">
			<div class="control-group">
				<label class="control-label">Device</label>
				<select
					class="device-select"
					:value="selectedDeviceIndex"
					:disabled="loading || isCapturing"
					@change="handleDeviceChange"
				>
					<option value="-1">Select Device</option>
					<option v-for="(device, index) in devices" :key="index" :value="index">
						{{ device }}
					</option>
				</select>
			</div>

			<div class="control-group">
				<label class="control-label">Gain {{ currentGain.toFixed(1) }}x</label>
				<input
					type="range"
					class="gain-slider"
					:value="currentGain"
					min="0.1"
					max="3.0"
					step="0.1"
					:disabled="loading"
					@input="handleGainChange"
				/>
			</div>

			<button
				class="control-btn"
				:class="{ active: isCapturing }"
				:disabled="loading || devices.length === 0"
				@click="$emit('capture-toggle')"
			>
				{{ isCapturing ? 'Stop' : 'Start' }}
			</button>
		</div>

		<!-- Spectrum Display -->
		<div class="spectrum-container" :class="{ active: isCapturing }">
			<div class="spectrum-bars">
				<div
					v-for="(value, index) in processedSpectrum"
					:key="index"
					class="spectrum-bar"
					:style="getSpectrumBarStyle(value)"
				></div>
			</div>
		</div>

		<!-- Metrics -->
		<div class="metrics-section">
			<div class="metric">
				<span class="metric-label">RMS</span>
				<div class="metric-bar">
					<div class="metric-fill" :style="{ width: `${spectrumRMS * 100}%` }"></div>
				</div>
				<span class="metric-value">{{ (spectrumRMS * 100).toFixed(0) }}%</span>
			</div>
			<div class="metric">
				<span class="metric-label">PEAK</span>
				<div class="metric-bar">
					<div class="metric-fill peak" :style="{ width: `${spectrumPeak * 100}%` }"></div>
				</div>
				<span class="metric-value">{{ (spectrumPeak * 100).toFixed(0) }}%</span>
			</div>
		</div>

		<!-- Error Display -->
		<div v-if="error" class="error-message">
			{{ error }}
		</div>
	</div>
</template>

<script setup lang="ts">
	import { computed } from 'vue';

	interface Props {
		isCapturing: boolean;
		devices: string[];
		currentGain: number;
		spectrum: number[];
		error: string | null;
		loading: boolean;
		selectedDeviceIndex: number | null;
		spectrumPeak: number;
		spectrumRMS: number;
	}

	interface Emits {
		(e: 'capture-toggle'): void;
		(e: 'device-change', deviceIndex: number): void;
		(e: 'gain-change', gain: number): void;
	}

	const props = defineProps<Props>();
	const emit = defineEmits<Emits>();

	// Gestionnaires d'événements avec typage sécurisé
	const handleDeviceChange = (event: Event): void => {
		const target = event.target as HTMLSelectElement;
		if (target && target.value !== null) {
			emit('device-change', Number(target.value));
		}
	};

	const handleGainChange = (event: Event): void => {
		const target = event.target as HTMLInputElement;
		if (target && target.value !== null) {
			emit('gain-change', Number(target.value));
		}
	};

	const processedSpectrum = computed(() => {
		if (!props.spectrum || props.spectrum.length === 0) {
			return Array(64).fill(0);
		}

		const normalized = props.spectrum.map((value) => {
			const logValue = Math.log10(Math.max(0.001, value * 10)) / 3;
			return Math.max(0, Math.min(1, logValue));
		});

		return normalized.slice(0, 64);
	});

	const getSpectrumBarStyle = (value: number) => {
		const height = Math.max(2, value * 100);
		return {
			height: `${height}%`,
			backgroundColor: '#c9d1d9',
			opacity: 0.7 + value * 0.3,
		};
	};
</script>

<style scoped>
	.audio-panel {
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

	.device-select {
		padding: 0.5rem 0.75rem;
		background: #21262d;
		border: 1px solid #30363d;
		border-radius: 4px;
		color: #c9d1d9;
		font-size: 0.875rem;
	}

	.device-select:focus {
		outline: none;
		border-color: #c9d1d9;
	}

	.device-select:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.gain-slider {
		width: 100%;
		height: 4px;
		background: #21262d;
		border-radius: 2px;
		outline: none;
		appearance: none;
		-webkit-appearance: none;
	}

	.gain-slider::-webkit-slider-thumb {
		appearance: none;
		-webkit-appearance: none;
		width: 16px;
		height: 16px;
		background: #c9d1d9;
		border-radius: 50%;
		cursor: pointer;
	}

	.gain-slider::-moz-range-thumb {
		width: 16px;
		height: 16px;
		background: #c9d1d9;
		border-radius: 50%;
		border: none;
		cursor: pointer;
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

	.spectrum-container {
		height: 120px;
		background: #161b22;
		border: 1px solid #21262d;
		border-radius: 4px;
		padding: 1rem;
		margin-bottom: 1rem;
		transition: border-color 0.3s ease;
	}

	.spectrum-container.active {
		border-color: #c9d1d9;
	}

	.spectrum-bars {
		display: flex;
		align-items: flex-end;
		height: 100%;
		gap: 1px;
	}

	.spectrum-bar {
		flex: 1;
		min-height: 2px;
		transition: all 0.1s ease;
		border-radius: 1px 1px 0 0;
	}

	.metrics-section {
		display: flex;
		gap: 1rem;
		margin-bottom: 1rem;
	}

	.metric {
		flex: 1;
		display: flex;
		align-items: center;
		gap: 0.75rem;
	}

	.metric-label {
		font-size: 0.75rem;
		font-weight: 600;
		color: #7d8590;
		min-width: 40px;
		text-transform: uppercase;
	}

	.metric-bar {
		flex: 1;
		height: 4px;
		background: #21262d;
		border-radius: 2px;
		overflow: hidden;
	}

	.metric-fill {
		height: 100%;
		background: #c9d1d9;
		transition: width 0.2s ease;
		border-radius: 2px;
	}

	.metric-fill.peak {
		background: #6e7681;
	}

	.metric-value {
		font-size: 0.75rem;
		font-weight: 600;
		color: #c9d1d9;
		min-width: 35px;
		text-align: right;
		font-family: monospace;
	}

	.error-message {
		padding: 0.75rem;
		background: #21262d;
		border: 1px solid #30363d;
		border-radius: 4px;
		color: #f85149;
		font-size: 0.875rem;
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
			gap: 0.75rem;
		}
	}
</style>
