<!-- src/components/PanelData.vue -->
<template>
	<div class="data-panel">
		<!-- Header -->
		<div class="panel-header">
			<div class="panel-info">
				<h2 class="panel-title">Audio Spectrum</h2>
				<div class="panel-specs">
					<span class="spec">{{ processedSpectrumData.length }} bands</span>
					<span v-if="displayFps > 0" class="spec">{{ displayFps }}fps</span>
				</div>
			</div>
			<div class="status-area">
				<div :class="['status', getStatusClass()]">
					<div class="status-dot"></div>
					<span>{{ getStatusText() }}</span>
				</div>
				<div class="spectrum-stats">
					<span class="stat-item">Peak: {{ maxSpectrumValue.toFixed(2) }}</span>
					<span class="stat-item">Updates: {{ spectrumCount }}</span>
				</div>
			</div>
		</div>

		<!-- Enhanced Audio Spectrum -->
		<div class="spectrum-section">
			<div class="spectrum-container" :class="{ streaming: isStreaming, offline: !isConnected }">
				<div class="spectrum-wrapper">
					<!-- Background grid -->
					<div class="spectrum-grid">
						<div v-for="i in 5" :key="i" class="grid-line" :style="{ top: `${(i - 1) * 20}%` }"></div>
					</div>

					<!-- Spectrum bars -->
					<div class="spectrum-bars">
						<div
							v-for="(value, index) in processedSpectrumData"
							:key="index"
							class="spectrum-bar-container"
						>
							<!-- Background bar -->
							<div class="spectrum-bar-bg"></div>

							<!-- Main bar -->
							<div class="spectrum-bar" :style="getSpectrumBarStyle(value, index)"></div>

							<!-- Peak indicator -->
							<div
								v-if="peakValues[index] > 0.1"
								class="spectrum-peak"
								:style="getPeakStyle(index)"
							></div>

							<!-- Glow effect for high values -->
							<div v-if="value > 0.7" class="spectrum-glow" :style="getGlowStyle(value, index)"></div>
						</div>
					</div>

					<!-- Frequency labels -->
					<div class="frequency-labels">
						<span class="freq-label">20Hz</span>
						<span class="freq-label">200Hz</span>
						<span class="freq-label">2kHz</span>
						<span class="freq-label">20kHz</span>
					</div>
				</div>

				<!-- RMS/Peak display -->
				<div class="spectrum-metrics">
					<div class="metric">
						<span class="metric-label">RMS</span>
						<div class="metric-bar">
							<div class="metric-fill rms" :style="{ width: `${rmsValue * 100}%` }"></div>
						</div>
						<span class="metric-value">{{ (rmsValue * 100).toFixed(1) }}%</span>
					</div>
					<div class="metric">
						<span class="metric-label">PEAK</span>
						<div class="metric-bar">
							<div class="metric-fill peak" :style="{ width: `${maxSpectrumValue * 100}%` }"></div>
						</div>
						<span class="metric-value">{{ (maxSpectrumValue * 100).toFixed(1) }}%</span>
					</div>
				</div>
			</div>
		</div>

		<!-- Debug section (optional) -->
		<div v-if="debugMode" class="debug-section">
			<div class="debug-header">
				<h3>Debug Info</h3>
				<button @click="clearDebugLogs" class="clear-btn">Clear</button>
			</div>
			<div class="debug-logs">
				<div v-for="(log, index) in debugLogs" :key="index" class="debug-log" :class="log.type">
					<span class="debug-time">{{ log.time }}</span>
					<span class="debug-message">{{ log.message }}</span>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
	import { computed, onMounted, onUnmounted, ref, watch } from 'vue';

	interface Props {
		spectrumData?: number[];
		fps?: number;
		isStreaming?: boolean;
		isConnected?: boolean;
		debugMode?: boolean;
	}

	const props = withDefaults(defineProps<Props>(), {
		spectrumData: () => [],
		fps: 0,
		isStreaming: false,
		isConnected: false,
		debugMode: false,
	});

	// Refs
	const spectrumCount = ref(0);
	const peakValues = ref<number[]>([]);
	const peakDecayRate = 0.95; // Peak decay rate
	const debugLogs = ref<Array<{ time: string; message: string; type: string }>>([]);

	// Animation frame
	let animationFrame: number | null = null;

	// Computed
	const displayFps = computed(() => props.fps || 0);

	const processedSpectrumData = computed(() => {
		if (!props.spectrumData || props.spectrumData.length === 0) {
			return Array(64).fill(0); // 64 bars par défaut
		}

		// Normalize and enhance the spectrum data
		const normalized = props.spectrumData.map((value) => {
			// Apply logarithmic scaling for better visual representation
			const logValue = Math.log10(Math.max(0.001, value * 10)) / 3;
			return Math.max(0, Math.min(1, logValue));
		});

		return normalized.slice(0, 64); // Limit to 64 bars max
	});

	const maxSpectrumValue = computed(() => {
		if (!processedSpectrumData.value || processedSpectrumData.value.length === 0) return 0;
		return Math.max(...processedSpectrumData.value);
	});

	const rmsValue = computed(() => {
		if (!processedSpectrumData.value || processedSpectrumData.value.length === 0) return 0;
		const sum = processedSpectrumData.value.reduce((acc, val) => acc + val * val, 0);
		return Math.sqrt(sum / processedSpectrumData.value.length);
	});

	// Debug
	const addDebugLog = (message: string, type: 'info' | 'success' | 'warning' | 'error' = 'info') => {
		if (!props.debugMode) return;

		const time = new Date().toLocaleTimeString();
		debugLogs.value.push({ time, message, type });

		if (debugLogs.value.length > 30) {
			debugLogs.value.shift();
		}

		console.log(`[PanelData] ${message}`);
	};

	const clearDebugLogs = () => {
		debugLogs.value = [];
	};

	// Status methods
	const getStatusClass = (): string => {
		if (!props.isConnected) return 'offline';
		if (!props.isStreaming) return 'ready';
		return 'live';
	};

	const getStatusText = (): string => {
		if (!props.isConnected) return 'OFFLINE';
		if (!props.isStreaming) return 'READY';
		return 'LIVE';
	};

	// Enhanced spectrum styling
	const getSpectrumBarStyle = (value: number, index: number) => {
		// Enhanced height calculation with minimum threshold
		const enhancedValue = Math.pow(value, 0.7); // Power curve for better visualization
		const height = Math.max(2, enhancedValue * 100);

		// Dynamic color based on frequency and amplitude
		const freqRatio = index / processedSpectrumData.value.length;
		let hue: number;

		if (freqRatio < 0.2) {
			// Bass: Red to Orange
			hue = 0 + freqRatio * 50;
		} else if (freqRatio < 0.6) {
			// Mids: Orange to Green
			hue = 50 + (freqRatio - 0.2) * 125;
		} else {
			// Highs: Green to Blue
			hue = 175 + (freqRatio - 0.6) * 85;
		}

		const saturation = 70 + value * 30;
		const lightness = 40 + value * 50;

		return {
			height: `${height}%`,
			background: `linear-gradient(to top,
            hsl(${hue}, ${saturation}%, ${lightness}%),
            hsl(${hue + 20}, ${saturation + 10}%, ${lightness + 20}%))`,
			boxShadow: value > 0.3 ? `0 0 ${value * 10}px hsl(${hue}, ${saturation}%, ${lightness}%)` : 'none',
			opacity: 0.8 + value * 0.2,
			transform: `scaleY(${0.8 + value * 0.2})`,
			transformOrigin: 'bottom',
		};
	};

	const getPeakStyle = (index: number) => {
		const peakValue = peakValues.value[index] || 0;
		const freqRatio = index / processedSpectrumData.value.length;
		const hue = freqRatio < 0.2 ? 0 : freqRatio < 0.6 ? 60 : 180;

		return {
			bottom: `${peakValue * 100}%`,
			backgroundColor: `hsl(${hue}, 90%, 70%)`,
			boxShadow: `0 0 8px hsl(${hue}, 90%, 70%)`,
		};
	};

	const getGlowStyle = (value: number, index: number) => {
		const freqRatio = index / processedSpectrumData.value.length;
		const hue = freqRatio < 0.2 ? 0 : freqRatio < 0.6 ? 60 : 180;

		return {
			height: `${value * 100}%`,
			background: `radial-gradient(ellipse at center,
            hsla(${hue}, 100%, 70%, ${value * 0.5}) 0%,
            transparent 70%)`,
		};
	};

	// Peak tracking
	const updatePeaks = () => {
		processedSpectrumData.value.forEach((value, index) => {
			if (!peakValues.value[index] || value > peakValues.value[index]) {
				peakValues.value[index] = value;
			} else {
				peakValues.value[index] *= peakDecayRate;
			}
		});
	};

	// Animation loop
	const animate = () => {
		updatePeaks();
		animationFrame = requestAnimationFrame(animate);
	};

	// Watchers
	watch(
		() => props.spectrumData,
		(newSpectrumData) => {
			if (newSpectrumData && newSpectrumData.length > 0) {
				spectrumCount.value++;

				// Initialize peak values array if needed
				if (peakValues.value.length !== processedSpectrumData.value.length) {
					peakValues.value = new Array(processedSpectrumData.value.length).fill(0);
				}
			}
		}
	);

	watch(
		() => props.isConnected,
		(isConnected) => {
			if (!isConnected) {
				spectrumCount.value = 0;
				peakValues.value = [];
			}
		}
	);

	// Lifecycle
	onMounted(() => {
		addDebugLog('PanelData mounted', 'info');
		animate();
	});

	onUnmounted(() => {
		addDebugLog('PanelData unmounted', 'info');
		if (animationFrame) {
			cancelAnimationFrame(animationFrame);
		}
	});
</script>

<style scoped>
	.data-panel {
		background: rgba(22, 27, 34, 0.95);
		border: 1px solid #30363d;
		border-radius: 16px;
		padding: 1.5rem;
		color: #f0f6fc;
		margin-bottom: 1.5rem;
		backdrop-filter: blur(8px);
		box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
	}

	/* Header */
	.panel-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 2rem;
	}

	.panel-title {
		margin: 0;
		font-size: 1.5rem;
		font-weight: 700;
		background: linear-gradient(135deg, #58a6ff 0%, #1f6feb 100%);
		-webkit-background-clip: text;
		-webkit-text-fill-color: transparent;
		background-clip: text;
	}

	.panel-specs {
		display: flex;
		gap: 0.5rem;
		margin-top: 0.5rem;
	}

	.spec {
		background: rgba(33, 38, 45, 0.8);
		color: #7d8590;
		padding: 0.25rem 0.75rem;
		border-radius: 20px;
		font-size: 0.75rem;
		border: 1px solid #30363d;
		backdrop-filter: blur(4px);
	}

	.status-area {
		display: flex;
		flex-direction: column;
		align-items: flex-end;
		gap: 0.75rem;
	}

	.status {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.5rem 1rem;
		border-radius: 20px;
		font-size: 0.75rem;
		font-weight: 600;
		text-transform: uppercase;
	}

	.status.offline {
		background: rgba(248, 81, 73, 0.1);
		color: #f85149;
		border: 1px solid #f85149;
	}

	.status.ready {
		background: rgba(88, 166, 255, 0.1);
		color: #58a6ff;
		border: 1px solid #58a6ff;
	}

	.status.live {
		background: rgba(35, 134, 54, 0.2);
		color: #2ea043;
		border: 1px solid #2ea043;
	}

	.status-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: currentColor;
		animation: pulse 1.5s infinite;
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

	.spectrum-stats {
		display: flex;
		gap: 1rem;
		color: #7d8590;
		font-size: 0.75rem;
		font-family: 'JetBrains Mono', monospace;
	}

	.stat-item {
		background: rgba(13, 17, 23, 0.8);
		padding: 0.25rem 0.5rem;
		border-radius: 8px;
		border: 1px solid #30363d;
	}

	/* Enhanced Spectrum */
	.spectrum-section {
		margin-bottom: 1.5rem;
	}

	.spectrum-container {
		background: linear-gradient(145deg, #0d1117 0%, #161b22 100%);
		border: 2px solid #30363d;
		border-radius: 16px;
		padding: 1.5rem;
		position: relative;
		overflow: hidden;
	}

	.spectrum-container.streaming {
		border-color: #2ea043;
		box-shadow: 0 0 30px rgba(46, 160, 67, 0.3);
	}

	.spectrum-container.offline {
		border-color: #f85149;
		box-shadow: 0 0 30px rgba(248, 81, 73, 0.2);
	}

	.spectrum-wrapper {
		position: relative;
		height: 200px;
		margin-bottom: 1rem;
	}

	.spectrum-grid {
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		pointer-events: none;
	}

	.grid-line {
		position: absolute;
		left: 0;
		right: 0;
		height: 1px;
		background: rgba(48, 54, 61, 0.3);
	}

	.spectrum-bars {
		display: flex;
		align-items: flex-end;
		height: 100%;
		gap: 1px;
		position: relative;
	}

	.spectrum-bar-container {
		flex: 1;
		position: relative;
		height: 100%;
		display: flex;
		align-items: flex-end;
	}

	.spectrum-bar-bg {
		position: absolute;
		bottom: 0;
		left: 0;
		right: 0;
		height: 2px;
		background: rgba(48, 54, 61, 0.5);
		border-radius: 1px;
	}

	.spectrum-bar {
		width: 100%;
		min-height: 2px;
		border-radius: 2px 2px 0 0;
		transition: all 0.1s cubic-bezier(0.4, 0, 0.2, 1);
		position: relative;
	}

	.spectrum-peak {
		position: absolute;
		left: 0;
		right: 0;
		height: 2px;
		border-radius: 1px;
		transition: bottom 0.2s ease-out;
	}

	.spectrum-glow {
		position: absolute;
		bottom: 0;
		left: -2px;
		right: -2px;
		border-radius: 4px;
		pointer-events: none;
	}

	.frequency-labels {
		position: absolute;
		bottom: -1.5rem;
		left: 0;
		right: 0;
		display: flex;
		justify-content: space-between;
		font-size: 0.65rem;
		color: #7d8590;
		font-family: 'JetBrains Mono', monospace;
	}

	.freq-label {
		background: rgba(13, 17, 23, 0.9);
		padding: 0.125rem 0.375rem;
		border-radius: 4px;
		border: 1px solid #30363d;
	}

	/* Spectrum Metrics */
	.spectrum-metrics {
		display: flex;
		gap: 1rem;
		padding-top: 1rem;
		border-top: 1px solid #30363d;
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
		font-family: 'JetBrains Mono', monospace;
	}

	.metric-bar {
		flex: 1;
		height: 6px;
		background: rgba(33, 38, 45, 0.8);
		border-radius: 3px;
		overflow: hidden;
		position: relative;
	}

	.metric-fill {
		height: 100%;
		border-radius: 3px;
		transition: width 0.2s ease;
	}

	.metric-fill.rms {
		background: linear-gradient(90deg, #2ea043 0%, #56d364 100%);
	}

	.metric-fill.peak {
		background: linear-gradient(90deg, #f85149 0%, #ff7b72 100%);
	}

	.metric-value {
		font-size: 0.75rem;
		font-weight: 600;
		color: #f0f6fc;
		min-width: 45px;
		text-align: right;
		font-family: 'JetBrains Mono', monospace;
	}

	/* Debug */
	.debug-section {
		background: rgba(13, 17, 23, 0.8);
		border: 1px solid #30363d;
		border-radius: 12px;
		padding: 1rem;
	}

	.debug-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1rem;
	}

	.debug-header h3 {
		margin: 0;
		font-size: 1rem;
	}

	.clear-btn {
		background: rgba(248, 81, 73, 0.1);
		color: #f85149;
		border: 1px solid #f85149;
		padding: 0.25rem 0.75rem;
		border-radius: 6px;
		font-size: 0.75rem;
		cursor: pointer;
		transition: background 0.2s ease;
	}

	.clear-btn:hover {
		background: rgba(248, 81, 73, 0.2);
	}

	.debug-logs {
		max-height: 200px;
		overflow-y: auto;
		font-family: 'JetBrains Mono', monospace;
		font-size: 0.75rem;
	}

	.debug-log {
		display: flex;
		gap: 0.5rem;
		padding: 0.25rem 0;
		border-bottom: 1px solid rgba(48, 54, 61, 0.2);
	}

	.debug-log:last-child {
		border-bottom: none;
	}

	.debug-log.success {
		color: #2ea043;
	}
	.debug-log.warning {
		color: #d29922;
	}
	.debug-log.error {
		color: #f85149;
	}
	.debug-log.info {
		color: #7d8590;
	}

	.debug-time {
		min-width: 80px;
		opacity: 0.7;
	}

	/* Responsive */
	@media (max-width: 768px) {
		.panel-header {
			flex-direction: column;
			gap: 1rem;
			align-items: flex-start;
		}

		.status-area {
			align-items: flex-start;
		}

		.spectrum-wrapper {
			height: 150px;
		}

		.spectrum-metrics {
			flex-direction: column;
			gap: 0.75rem;
		}
	}

	@media (max-width: 480px) {
		.data-panel {
			padding: 1rem;
		}

		.spectrum-wrapper {
			height: 120px;
		}
	}

	/* Accessibility */
	@media (prefers-reduced-motion: reduce) {
		.spectrum-bar,
		.spectrum-peak,
		.metric-fill {
			transition: none;
		}

		.status-dot {
			animation: none;
		}
	}
</style>
