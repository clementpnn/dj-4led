<template>
	<div class="audio-panel">
		<div class="panel-header">
			<h2>Audio Spectrum</h2>
			<div class="status-indicator" :class="{ active: audioStore.isHealthy, error: audioStore.state.error }">
				{{ audioStore.state.error ? '❌' : audioStore.isHealthy ? '🟢' : '⚪' }}
			</div>
		</div>

		<!-- Controls -->
		<div class="controls-section">
			<div class="control-group">
				<label class="control-label">Device ({{ audioStore.state.devices.length }} found)</label>
				<select
					class="device-select"
					:value="audioStore.selectedDeviceIndex ?? -1"
					:disabled="audioStore.loading || audioStore.state.isCapturing"
					@change="handleDeviceChange"
				>
					<option value="-1">Auto-detect</option>
					<option v-for="(device, index) in audioStore.state.devices" :key="index" :value="index">
						{{ device }}
					</option>
				</select>
			</div>

			<div class="control-group">
				<label class="control-label">Gain {{ audioStore.state.currentGain.toFixed(1) }}x</label>
				<input
					type="range"
					class="gain-slider"
					:value="audioStore.state.currentGain"
					min="0.1"
					max="3.0"
					step="0.1"
					:disabled="audioStore.loading"
					@input="handleGainChange"
				/>
			</div>

			<div class="button-group">
				<button
					class="control-btn"
					:class="{ active: audioStore.state.isCapturing, loading: audioStore.loading }"
					:disabled="audioStore.loading"
					@click="handleCaptureToggle"
				>
					<span v-if="audioStore.loading">●●●</span>
					<span v-else>{{ audioStore.state.isCapturing ? 'Stop' : 'Start' }}</span>
				</button>

				<button
					class="control-btn secondary"
					:disabled="audioStore.loading"
					title="Refresh audio devices"
					@click="handleRefreshDevices"
				>
					🔄
				</button>
			</div>
		</div>

		<!-- Spectrum Display -->
		<div class="spectrum-container" :class="{ active: audioStore.state.isCapturing }">
			<div class="spectrum-bars">
				<div
					v-for="(value, index) in processedSpectrum"
					:key="index"
					class="spectrum-bar"
					:style="getSpectrumBarStyle(value, index)"
				></div>
			</div>
			<div v-if="!audioStore.state.isCapturing" class="spectrum-overlay">
				<span>{{
					audioStore.state.devices.length === 0
						? 'No audio devices found - try refreshing'
						: 'Click Start to begin audio capture'
				}}</span>
			</div>
		</div>

		<!-- Metrics -->
		<div class="metrics-section">
			<div class="metric">
				<span class="metric-label">RMS</span>
				<div class="metric-bar">
					<div
						class="metric-fill"
						:style="{ width: `${Math.min(100, audioStore.spectrumRMS * 100)}%` }"
					></div>
				</div>
				<span class="metric-value">{{ (audioStore.spectrumRMS * 100).toFixed(0) }}%</span>
			</div>
			<div class="metric">
				<span class="metric-label">PEAK</span>
				<div class="metric-bar">
					<div
						class="metric-fill peak"
						:style="{ width: `${Math.min(100, audioStore.spectrumPeak * 100)}%` }"
					></div>
				</div>
				<span class="metric-value">{{ (audioStore.spectrumPeak * 100).toFixed(0) }}%</span>
			</div>
			<div class="metric">
				<span class="metric-label">SIGNAL</span>
				<div class="metric-bar">
					<div class="metric-fill signal" :style="{ width: `${signalStrength}%` }"></div>
				</div>
				<span class="metric-value">{{ signalStrength.toFixed(0) }}%</span>
			</div>
		</div>

		<!-- Error Display -->
		<div v-if="audioStore.state.error" class="error-message">
			<span class="error-icon">⚠️</span>
			{{ audioStore.state.error }}
			<button class="error-close" @click="handleClearError">×</button>
		</div>
	</div>
</template>

<script setup lang="ts">
	import { computed } from 'vue';

	import { useAudio } from '@/composables/useAudio';
	import { useAudioStore } from '@/stores/audio';

	// Store - pour récupérer les données
	const audioStore = useAudioStore();

	// Composable - pour la logique et les actions uniquement
	const { startCapture, stopCapture, setGain, getDevices, clearError } = useAudio();

	// ===== COMPUTED =====

	const processedSpectrum = computed(() => {
		// Toujours retourner un spectre vide si pas en capture
		if (!audioStore.state.isCapturing || !audioStore.state.spectrum || audioStore.state.spectrum.length === 0) {
			return Array(64).fill(0);
		}

		// Le backend envoie déjà un spectrum traité avec FFT et normalisé
		// On applique juste le gain et on s'assure d'avoir 64 valeurs
		const normalized = audioStore.state.spectrum.map((value) => {
			// Appliquer le gain avec limitation
			const gainedValue = value * audioStore.state.currentGain;
			// Limiter à [0, 1] avec une courbe légèrement compressée
			return Math.max(0, Math.min(1, gainedValue * 0.8));
		});

		// S'assurer qu'on a exactement 64 barres
		if (normalized.length >= 64) {
			return normalized.slice(0, 64);
		} else {
			// Interpoler si on a moins de 64 valeurs
			const result = Array(64).fill(0);
			const step = normalized.length / 64;
			for (let i = 0; i < 64; i++) {
				const sourceIndex = Math.floor(i * step);
				result[i] = normalized[sourceIndex] || 0;
			}
			return result;
		}
	});

	const signalStrength = computed(() => {
		if (processedSpectrum.value.length === 0) return 0;

		// Calculer la force du signal basée sur le nombre de bins actifs
		const activeBins = processedSpectrum.value.filter((v) => v > 0.05).length;
		const strength = (activeBins / processedSpectrum.value.length) * 100;
		return Math.min(100, strength);
	});

	// ===== HANDLERS =====

	const handleDeviceChange = async (event: Event): Promise<void> => {
		const target = event.target as HTMLSelectElement;
		if (!target || target.value === null) return;

		const deviceIndex = Number(target.value);

		try {
			if (deviceIndex >= 0) {
				audioStore.setSelectedDevice(deviceIndex);
				console.log(`Audio device selected: ${audioStore.state.devices[deviceIndex]}`);
			} else {
				audioStore.setSelectedDevice(null);
				console.log('Audio device set to auto-detect');
			}
		} catch (error) {
			console.error(`Device selection error: ${error}`);
		}
	};

	const handleGainChange = async (event: Event): Promise<void> => {
		const target = event.target as HTMLInputElement;
		if (!target || target.value === null) return;

		const newGain = Number(target.value);
		try {
			const result = await setGain(newGain);
			if (!result.success) {
				console.error(`Failed to set gain: ${result.message}`);
			}
		} catch (error) {
			console.error(`Gain change error: ${error}`);
		}
	};

	const handleCaptureToggle = async (): Promise<void> => {
		try {
			const result = audioStore.state.isCapturing ? await stopCapture() : await startCapture();
			console.log('Audio capture toggle result:', result);
		} catch (error) {
			console.error(`Audio capture error: ${error}`);
		}
	};

	const handleRefreshDevices = async (): Promise<void> => {
		try {
			const result = await getDevices();
			console.log('Audio devices refresh result:', result);
		} catch (error) {
			console.error(`Device refresh error: ${error}`);
		}
	};

	const handleClearError = (): void => {
		clearError();
	};

	// ===== UTILITIES =====

	const getSpectrumBarStyle = (value: number, index: number) => {
		const height = Math.max(2, value * 100);

		// Couleurs différentes selon les fréquences
		let hue: number;
		if (index < 8) {
			// Basses: rouge à orange
			hue = 0 + (index / 8) * 30;
		} else if (index < 24) {
			// Médiums: orange à jaune
			hue = 30 + ((index - 8) / 16) * 30;
		} else if (index < 48) {
			// Médiums-aigus: jaune à vert
			hue = 60 + ((index - 24) / 24) * 60;
		} else {
			// Aigus: vert à cyan
			hue = 120 + ((index - 48) / 16) * 60;
		}

		const saturation = 70 + value * 30;
		const lightness = 50 + value * 40;

		return {
			height: `${height}%`,
			backgroundColor: `hsl(${hue}, ${saturation}%, ${lightness}%)`,
			opacity: 0.6 + value * 0.4,
			transition: 'height 0.08s ease-out, background-color 0.1s ease',
			boxShadow: value > 0.7 ? `0 0 6px hsl(${hue}, ${saturation}%, ${lightness}%)` : 'none',
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
		display: flex;
		align-items: center;
		justify-content: space-between;
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
		font-size: 0.875rem;
		transition: all 0.3s ease;
	}

	.status-indicator.active {
		animation: pulse 2s infinite;
	}

	.status-indicator.error {
		animation: shake 0.5s ease-in-out;
	}

	@keyframes pulse {
		0%,
		100% {
			opacity: 1;
		}
		50% {
			opacity: 0.6;
		}
	}

	@keyframes shake {
		0%,
		100% {
			transform: translateX(0);
		}
		25% {
			transform: translateX(-2px);
		}
		75% {
			transform: translateX(2px);
		}
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

	.button-group {
		display: flex;
		gap: 0.5rem;
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
		transition: border-color 0.2s ease;
		min-width: 200px;
	}

	.device-select:focus {
		outline: none;
		border-color: #58a6ff;
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
		background: #58a6ff;
		border-radius: 50%;
		cursor: pointer;
		transition: background-color 0.2s ease;
	}

	.gain-slider::-webkit-slider-thumb:hover {
		background: #79c0ff;
	}

	.gain-slider::-moz-range-thumb {
		width: 16px;
		height: 16px;
		background: #58a6ff;
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

	.control-btn.secondary {
		min-width: 50px;
		padding: 0.75rem;
	}

	.control-btn:hover:not(:disabled) {
		background: #30363d;
		border-color: #58a6ff;
	}

	.control-btn.active {
		background: #238636;
		color: #ffffff;
		border-color: #2ea043;
	}

	.control-btn.loading {
		opacity: 0.8;
		cursor: not-allowed;
	}

	.control-btn.loading span {
		animation: loading-dots 1.5s infinite;
	}

	@keyframes loading-dots {
		0%,
		20% {
			opacity: 0.2;
		}
		50% {
			opacity: 1;
		}
		80%,
		100% {
			opacity: 0.2;
		}
	}

	.control-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.spectrum-container {
		position: relative;
		height: 140px;
		background: #161b22;
		border: 1px solid #21262d;
		border-radius: 4px;
		padding: 1rem;
		margin-bottom: 1rem;
		transition: border-color 0.3s ease;
		overflow: hidden;
	}

	.spectrum-container.active {
		border-color: #238636;
		box-shadow: 0 0 0 1px #238636;
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
		border-radius: 1px 1px 0 0;
	}

	.spectrum-overlay {
		position: absolute;
		top: 50%;
		left: 50%;
		transform: translate(-50%, -50%);
		color: #7d8590;
		font-size: 0.875rem;
		text-align: center;
		pointer-events: none;
	}

	.metrics-section {
		display: flex;
		gap: 1rem;
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
		min-width: 45px;
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
		background: #238636;
		transition: width 0.2s ease;
		border-radius: 2px;
	}

	.metric-fill.peak {
		background: #fd7e14;
	}

	.metric-fill.signal {
		background: #58a6ff;
	}

	.metric-value {
		font-size: 0.75rem;
		font-weight: 600;
		color: #c9d1d9;
		min-width: 35px;
		text-align: right;
		font-family: 'Courier New', monospace;
	}

	.error-message {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.75rem;
		background: #21262d;
		border: 1px solid #f85149;
		border-radius: 4px;
		color: #ffa198;
		font-size: 0.875rem;
		margin-top: 1rem;
	}

	.error-icon {
		font-size: 1rem;
	}

	.error-close {
		background: none;
		border: none;
		color: #ffa198;
		cursor: pointer;
		font-size: 1.2rem;
		padding: 0;
		margin-left: auto;
		transition: opacity 0.2s ease;
	}

	.error-close:hover {
		opacity: 0.7;
	}

	@media (max-width: 768px) {
		.controls-section {
			flex-direction: column;
			align-items: stretch;
		}

		.button-group {
			margin-top: 0.5rem;
		}

		.metrics-section {
			flex-direction: column;
			gap: 0.75rem;
		}
	}
</style>
