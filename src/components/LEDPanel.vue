<template>
	<div class="led-panel">
		<!-- Header -->
		<div class="panel-header">
			<h2>LED Output</h2>
			<div class="header-info">
				<span class="matrix-badge">{{ ledStore.matrixSize }}</span>
				<div class="status-badge" :class="{ running: ledStore.isRunning }">
					{{ ledStore.isRunning ? 'RUNNING' : 'STOPPED' }}
				</div>
			</div>
		</div>

		<!-- Controls -->
		<div class="controls-section">
			<select
				class="mode-select"
				:value="ledStore.currentMode"
				:disabled="ledStore.loading || isChanging"
				@change="handleModeChange"
			>
				<option value="simulator">Simulator</option>
				<option value="production">Production</option>
			</select>

			<button
				class="control-btn"
				:class="{ active: ledStore.isRunning }"
				:disabled="ledStore.loading || isChanging"
				@click="handleOutputToggle"
			>
				{{ ledStore.isRunning ? '⏹ Stop' : '▶ Start' }}
			</button>
		</div>

		<!-- Canvas Display -->
		<div class="display-section">
			<div class="display-header">
				<h3>Live Matrix</h3>
				<div class="display-stats">
					<span class="stat">{{ framesStore.stats.fps }} FPS</span>
					<span class="stat" :class="latencyClass">{{ latency }}ms</span>
				</div>
			</div>

			<div class="display-container" :class="{ active: ledStore.isRunning }">
				<canvas
					v-show="showCanvas"
					ref="ledCanvas"
					:width="canvasSize"
					:height="canvasSize"
					class="led-canvas"
					:style="{ transform: `scale(${zoom})` }"
				/>
				<div v-if="!showCanvas" class="no-display">
					{{ displayMessage }}
				</div>
			</div>
		</div>

		<!-- Metrics -->
		<div class="metrics-section">
			<div class="metric">
				<span class="metric-label">Frames</span>
				<span class="metric-value">{{ formatNumber(framesStore.stats.frameCount) }}</span>
			</div>
			<div class="metric">
				<span class="metric-label">Success</span>
				<span class="metric-value">{{ framesStore.metrics.successRate.toFixed(1) }}%</span>
			</div>
			<div class="metric">
				<span class="metric-label">Controllers</span>
				<span class="metric-value">{{ ledStore.controllerCount }}</span>
			</div>
			<div class="metric">
				<span class="metric-label">Latency</span>
				<span class="metric-value" :class="latencyClass">{{ latency }}ms</span>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
	import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';

	import { useFrames } from '@/composables/useFrames';
	import { useLED } from '@/composables/useLED';
	import { useFramesStore } from '@/stores/frames';
	import { useLEDStore } from '@/stores/led';

	// Stores
	const ledStore = useLEDStore();
	const framesStore = useFramesStore();

	// Composables - initialisation complète
	const ledComposable = useLED();
	const framesComposable = useFrames();

	// Destructuring des méthodes dont on a besoin
	const { startOutput, stopOutput, getStatus } = ledComposable;
	const { autoRefresh, getCurrentFrame } = framesComposable;

	// State
	const ledCanvas = ref<HTMLCanvasElement>();
	const canvasSize = ref(512);
	const zoom = ref(1);
	const latency = ref(0);
	const isChanging = ref(false);
	let animationId: number | null = null;

	// Computed
	const showCanvas = computed(() => framesStore.hasCurrentFrame && ledStore.isRunning);

	const displayMessage = computed(() => {
		if (ledStore.loading || isChanging.value) return 'Loading...';
		if (!ledStore.isRunning) return 'Output stopped';
		if (ledStore.isRunning && !framesStore.hasCurrentFrame) return 'Waiting for frames...';
		return 'Ready';
	});

	const latencyClass = computed(() => {
		if (latency.value < 50) return 'good';
		if (latency.value < 100) return 'medium';
		return 'high';
	});

	// Methods
	const formatNumber = (num: number): string => {
		if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`;
		if (num >= 1000) return `${(num / 1000).toFixed(1)}K`;
		return num.toString();
	};

	const drawFrame = () => {
		if (!ledCanvas.value || !framesStore.currentFrame) return;

		const ctx = ledCanvas.value.getContext('2d', { alpha: false });
		if (!ctx) return;

		const frame = framesStore.currentFrame;
		const pixelSize = Math.floor(canvasSize.value / frame.width);
		const gap = 1;

		// Clear canvas
		ctx.fillStyle = '#0d1117';
		ctx.fillRect(0, 0, canvasSize.value, canvasSize.value);

		// Draw pixels with brightness
		const brightness = ledStore.brightness;

		for (let y = 0; y < frame.height; y++) {
			for (let x = 0; x < frame.width; x++) {
				const i = (y * frame.width + x) * 3;
				const r = Math.floor((frame.data[i] || 0) * brightness);
				const g = Math.floor((frame.data[i + 1] || 0) * brightness);
				const b = Math.floor((frame.data[i + 2] || 0) * brightness);

				if (r || g || b) {
					ctx.fillStyle = `rgb(${r},${g},${b})`;
					ctx.fillRect(x * pixelSize + gap, y * pixelSize + gap, pixelSize - gap * 2, pixelSize - gap * 2);
				}
			}
		}
	};

	const animate = () => {
		drawFrame();
		if (ledStore.isRunning && autoRefresh.value) {
			animationId = requestAnimationFrame(animate);
		}
	};

	const stopAnimation = () => {
		if (animationId) {
			cancelAnimationFrame(animationId);
			animationId = null;
		}
	};

	// Handlers
	const handleModeChange = async (event: Event) => {
		const mode = (event.target as HTMLSelectElement).value as 'simulator' | 'production';
		if (isChanging.value || mode === ledStore.currentMode) return;

		isChanging.value = true;
		try {
			if (ledStore.isRunning) {
				await stopOutput();
				stopAnimation();
			}
			// Attendre un peu avant de redémarrer
			await new Promise((resolve) => setTimeout(resolve, 100));
			await startOutput(mode);

			// Attendre que le LED soit vraiment démarré
			await new Promise((resolve) => setTimeout(resolve, 200));

			if (autoRefresh.value) {
				await getCurrentFrame();
				animate();
			}
		} finally {
			isChanging.value = false;
		}
	};

	const handleOutputToggle = async () => {
		if (isChanging.value) return;

		isChanging.value = true;
		try {
			if (ledStore.isRunning) {
				stopAnimation();
				await stopOutput();
			} else {
				await startOutput(ledStore.currentMode);

				// Attendre que le système soit prêt
				await new Promise((resolve) => setTimeout(resolve, 200));

				// Récupérer la première frame
				if (autoRefresh.value) {
					await getCurrentFrame();
					animate();
				}
			}
		} finally {
			isChanging.value = false;
		}
	};

	// Watchers
	watch(
		() => framesStore.currentFrame,
		() => {
			nextTick(() => drawFrame());
		}
	);

	watch(
		() => framesStore.stats.lastFrameTime,
		(newTime) => {
			if (newTime > 0) {
				const newLatency = Date.now() - newTime;
				latency.value = Math.min(999, Math.max(0, newLatency));
			}
		}
	);

	watch(
		() => ledStore.isRunning,
		async (running) => {
			if (running) {
				// Attendre un peu que le système soit prêt
				await new Promise((resolve) => setTimeout(resolve, 100));

				if (autoRefresh.value) {
					await getCurrentFrame();
					animate();
				}
			} else {
				stopAnimation();
			}
		}
	);

	watch(
		() => autoRefresh.value,
		(enabled) => {
			if (enabled && ledStore.isRunning) {
				getCurrentFrame().then(() => animate());
			} else {
				stopAnimation();
			}
		}
	);

	// Update canvas when brightness changes
	watch(
		() => ledStore.brightness,
		() => {
			if (framesStore.hasCurrentFrame) {
				drawFrame();
			}
		}
	);

	// Lifecycle
	onMounted(async () => {
		// Initialize canvas size based on matrix
		const [width] = ledStore.matrixSize.split('x').map(Number);
		if (width) {
			const scale = Math.min(512 / width, 512 / width);
			canvasSize.value = Math.floor(width * scale);
		}

		// Get initial status
		await getStatus();

		// Start animation if already running
		if (ledStore.isRunning && autoRefresh.value) {
			await getCurrentFrame();
			animate();
		}
	});

	onUnmounted(() => {
		stopAnimation();
	});
</script>

<style scoped>
	.led-panel {
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
	}

	.header-info {
		display: flex;
		align-items: center;
		gap: 0.75rem;
	}

	.matrix-badge {
		padding: 0.25rem 0.5rem;
		background: #161b22;
		border: 1px solid #21262d;
		border-radius: 4px;
		font-size: 0.75rem;
		font-family: monospace;
		color: #7d8590;
	}

	.status-badge {
		padding: 0.375rem 0.75rem;
		background: #161b22;
		border: 1px solid #21262d;
		border-radius: 4px;
		font-size: 0.75rem;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.025em;
		color: #7d8590;
		transition: all 0.3s ease;
	}

	.status-badge.running {
		color: #3fb950;
		border-color: #238636;
	}

	/* Controls */
	.controls-section {
		display: flex;
		gap: 1rem;
		margin-bottom: 1.5rem;
	}

	.mode-select {
		flex: 1;
		padding: 0.5rem 0.75rem;
		background: #161b22;
		border: 1px solid #21262d;
		border-radius: 4px;
		color: #c9d1d9;
		font-size: 0.875rem;
		cursor: pointer;
	}

	.mode-select:focus {
		outline: none;
		border-color: #58a6ff;
	}

	.mode-select:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.control-btn {
		padding: 0.5rem 1.25rem;
		background: #21262d;
		border: 1px solid #30363d;
		border-radius: 4px;
		color: #c9d1d9;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.control-btn:hover:not(:disabled) {
		background: #30363d;
	}

	.control-btn.active {
		background: #238636;
		color: white;
		border-color: #2ea043;
	}

	.control-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	/* Display */
	.display-section {
		margin-bottom: 1.5rem;
	}

	.display-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1rem;
	}

	.display-header h3 {
		margin: 0;
		font-size: 1rem;
		font-weight: 600;
	}

	.display-stats {
		display: flex;
		gap: 0.75rem;
	}

	.stat {
		padding: 0.25rem 0.5rem;
		background: #161b22;
		border: 1px solid #21262d;
		border-radius: 4px;
		font-size: 0.75rem;
		font-weight: 600;
		font-family: monospace;
		color: #7d8590;
	}

	.stat.good {
		color: #3fb950;
	}

	.stat.medium {
		color: #d29922;
	}

	.stat.high {
		color: #f85149;
	}

	.display-container {
		height: 300px;
		background: #161b22;
		border: 1px solid #21262d;
		border-radius: 4px;
		display: flex;
		align-items: center;
		justify-content: center;
		position: relative;
		overflow: hidden;
		transition: border-color 0.3s ease;
	}

	.display-container.active {
		border-color: #238636;
	}

	.led-canvas {
		image-rendering: pixelated;
		image-rendering: -moz-crisp-edges;
		image-rendering: crisp-edges;
		cursor: crosshair;
		transition: transform 0.3s ease;
	}

	.no-display {
		color: #7d8590;
		font-size: 0.875rem;
	}

	/* Metrics */
	.metrics-section {
		display: flex;
		gap: 1rem;
		margin-bottom: 1.5rem;
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

	.metric-value.good {
		color: #3fb950;
	}

	.metric-value.medium {
		color: #d29922;
	}

	.metric-value.high {
		color: #f85149;
	}

	/* Responsive */
	@media (max-width: 768px) {
		.led-panel {
			padding: 1rem;
		}

		.controls-section {
			flex-direction: column;
		}

		.metrics-section {
			display: grid;
			grid-template-columns: 1fr 1fr;
			gap: 0.75rem;
		}

		.display-container {
			height: 200px;
		}
	}

	@media (max-width: 480px) {
		.metrics-section {
			grid-template-columns: 1fr;
		}
	}
</style>
