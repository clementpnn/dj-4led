<template>
	<div id="app" class="app-container">
		<!-- Header avec status global -->
		<Header
			:is-connected="system.isOnline"
			:fps="frames.frameRate"
			:is-streaming="audio.state.isCapturing && led.isRunning"
			:stream-loading="system.loading"
			:is-stream-healthy="system.isHealthy"
			:loading="system.loading"
			:ping-ms="0"
			@connect="handleQuickStart"
			@disconnect="handleShutdown"
			@ping="handleHealthCheck"
			@stream-toggle="handleStreamToggle"
		/>

		<!-- Navigation principale -->
		<nav class="main-nav">
			<div class="nav-container">
				<button
					v-for="tab in tabs"
					:key="tab.id"
					@click="activeTab = tab.id"
					:class="['nav-tab', { active: activeTab === tab.id }]"
				>
					<span class="tab-icon">{{ tab.icon }}</span>
					<span class="tab-label">{{ tab.label }}</span>
					<span v-if="tab.badge" class="tab-badge">{{ tab.badge }}</span>
				</button>
			</div>
		</nav>

		<!-- Contenu principal -->
		<main class="main-content">
			<div class="content-grid">
				<!-- Colonne principale -->
				<div class="main-column">
					<!-- Audio Panel -->
					<div v-show="activeTab === 'audio'" class="tab-content">
						<AudioSpectrumPanel
							:is-capturing="audio.state.isCapturing"
							:devices="audio.state.devices"
							:current-gain="audio.state.currentGain"
							:spectrum="audio.state.spectrum"
							:error="audio.state.error"
							:loading="audio.loading"
							:selected-device-index="selectedDeviceIndex"
							:spectrum-peak="audio.spectrumPeak"
							:spectrum-r-m-s="audio.spectrumRMS"
							@capture-toggle="handleAudioToggle"
							@device-change="handleDeviceChange"
							@gain-change="handleGainChange"
						/>
					</div>

					<!-- Effects Panel -->
					<div v-show="activeTab === 'effects'" class="tab-content">
						<EffectsPanel
							:available-effects="effects.availableEffects"
							:current-effect="effects.currentEffect"
							:effect-info="effects.effectInfo"
							:loading="effects.loading"
							:current-effect-name="effects.currentEffectName"
							:is-transitioning="effects.isTransitioning"
							:transition-progress="effects.transitionProgress"
							:effects-by-category="effects.effectsByCategory"
							@effect-change="handleEffectChange"
							@refresh-effects="handleRefreshEffects"
							@get-effect-info="handleGetEffectInfo"
						/>
					</div>

					<!-- Colors Panel -->
					<div v-show="activeTab === 'colors'" class="tab-content">
						<ColorsPanel
							:current-mode="colors.currentMode"
							:custom-color="colors.customColor"
							:available-modes="colors.availableModes"
							:loading="colors.loading"
							:color-preview-style="colors.colorPreviewStyle"
							:hex-color="colors.hexColor"
							:current-mode-info="colors.currentModeInfo"
							:is-custom-mode="colors.isCustomMode"
							@mode-change="handleColorModeChange"
							@color-change="handleCustomColorChange"
							@apply-custom-color="handleApplyCustomColor"
						/>
					</div>

					<!-- LED Panel -->
					<div v-show="activeTab === 'led'" class="tab-content">
						<LEDPanel
							:is-running="led.isRunning"
							:brightness="led.brightness"
							:current-mode="led.currentMode"
							:loading="led.loading"
							:current-frame="frames.currentFrame"
							:frame-image-url="currentFrameImageUrl"
							:frame-rate="frames.frameRate.value"
							:frame-count="frames.frameCount"
							:stats="frames.stats"
							:metrics="frames.metrics"
							:controller-count="led.controllerCount"
							:connected-controllers="led.connectedControllers"
							:frame-size="led.frameSize"
							:matrix-size="led.matrixSize"
							:is-healthy="led.isHealthy"
							:has-current-frame="frames.hasCurrentFrame"
							@output-toggle="handleLEDToggle"
							@brightness-change="handleBrightnessChange"
							@mode-change="handleLEDModeChange"
						/>
					</div>

					<!-- Presets Panel -->
					<div v-show="activeTab === 'presets'" class="tab-content">
						<PresetsPanel
							:all-presets="presets.allPresets"
							:custom-presets="presets.customPresets"
							:current-preset="presets.currentPreset"
							:loading="presets.loading"
							@apply-preset="handleApplyPreset"
							@create-preset="handleCreatePreset"
							@duplicate-preset="handleDuplicatePreset"
							@delete-preset="handleDeletePreset"
							@export-presets="handleExportPresets"
							@import-presets="handleImportPresets"
						/>
					</div>

					<!-- System Panel -->
					<div v-show="activeTab === 'system'" class="tab-content">
						<SystemStatusPanel
							:stats="system.stats"
							:health="system.health"
							:is-online="system.isOnline"
							:loading="system.loading"
							:connection-quality="system.connectionQuality"
							:uptime="systemUptime"
							@health-check="handleHealthCheck"
							@refresh-stats="handleRefreshStats"
						/>
					</div>
				</div>

				<!-- Sidebar -->
				<aside class="sidebar">
					<!-- Status Data Panel -->
					<PanelData
						:spectrum-data="audio.state.spectrum"
						:fps="frames.frameRate.value"
						:is-streaming="audio.state.isCapturing && led.isRunning"
						:is-connected="system.isOnline"
						:debug-mode="debugMode"
					/>

					<!-- Console Logs -->
					<div class="console-container">
						<Terminal
							:logs="logs.filteredLogs"
							:auto-scroll="logs.autoScroll.value"
							@toggle-auto-scroll="logs.toggleAutoScroll"
							@export-logs="logs.exportLogs"
							@clear-logs="logs.clearLogs"
						/>
					</div>
				</aside>
			</div>
		</main>

		<!-- Debug toggle (développement uniquement) -->
		<button v-if="isDev" @click="debugMode = !debugMode" class="debug-toggle" :class="{ active: debugMode }">
			🐛
		</button>

		<!-- Notifications toast (optionnel) -->
		<div v-if="notification" class="notification" :class="notification.type">
			{{ notification.message }}
		</div>
	</div>
</template>

<script setup lang="ts">
	import { computed, onMounted, onUnmounted, ref } from 'vue';

	// Composables
	import { useAudio } from './composables/useAudio';
	import { useColors } from './composables/useColors';
	import { useEffects } from './composables/useEffects';
	import { useFrames } from './composables/useFrames';
	import { useLED } from './composables/useLED';
	import { useLogs } from './composables/useLogs';
	import { usePresets } from './composables/usePresets';
	import { useSystem } from './composables/useSystem';

	// Components
	import AudioSpectrumPanel from './components/AudioSpectrumPanel.vue';
	import ColorsPanel from './components/ColorsPanel.vue';
	import EffectsPanel from './components/EffectsPanel.vue';
	import Header from './components/Header.vue';
	import LEDPanel from './components/LEDPanel.vue';
	import PanelData from './components/PanelData.vue';
	import PresetsPanel from './components/PresetsPanel.vue';
	import SystemStatusPanel from './components/SystemStatusPanel.vue';
	import Terminal from './components/Terminal.vue';

	// Types
	import type { CustomColor } from './types';

	// State
	const activeTab = ref('audio');
	const selectedDeviceIndex = ref<number>(-1);
	const debugMode = ref(false);
	const isDev = ref(import.meta.env.DEV);
	const startTime = ref(Date.now());
	const notification = ref<{ message: string; type: string } | null>(null);

	// Composables
	const audio = useAudio();
	const colors = useColors();
	const effects = useEffects();
	const frames = useFrames();
	const led = useLED();
	const logs = useLogs();
	const presets = usePresets();
	const system = useSystem();

	// Configuration des onglets
	const tabs = computed(() => [
		{
			id: 'audio',
			label: 'Audio',
			icon: '🎧',
			badge: audio.state.isCapturing ? '●' : null,
		},
		{
			id: 'effects',
			label: 'Effects',
			icon: '🎇',
			badge: effects.currentEffect ? '●' : null,
		},
		{
			id: 'colors',
			label: 'Colors',
			icon: '🌈',
			badge: null,
		},
		{
			id: 'led',
			label: 'LED Output',
			icon: '💡',
			badge: led.isRunning ? '●' : null,
		},
		{
			id: 'presets',
			label: 'Presets',
			icon: '🎛️',
			badge: presets.customPresets.length > 0 ? presets.customPresets.length.toString() : null,
		},
		{
			id: 'system',
			label: 'System',
			icon: '📊',
			badge: system.health.status === 'healthy' ? null : '⚠️',
		},
	]);

	// Computed
	const systemUptime = computed(() => Date.now() - startTime.value);

	const currentFrameImageUrl = computed(() => {
		if (frames.currentFrame) {
			return frames.frameToImageUrl(frames.currentFrame);
		}
		return '';
	});

	// Handlers pour Effects
	const handleEffectChange = async (effectId: number): Promise<void> => {
		const result = await effects.setEffect(effectId);
		logs.logAction('Effect Change', result, 'effects');
		showNotification(result.message, result.success ? 'success' : 'error');
	};

	const handleRefreshEffects = async (): Promise<void> => {
		const result = await effects.getAvailableEffects();
		logs.logAction('Refresh Effects', result, 'effects');
		showNotification(result.message, result.success ? 'success' : 'error');
	};

	const handleGetEffectInfo = async (effectId: number): Promise<void> => {
		const result = await effects.getEffectInfo(effectId);
		if (!result.success) {
			logs.logAction('Get Effect Info', result, 'effects');
		}
	};
	const handleAudioToggle = async (): Promise<void> => {
		const result = audio.state.isCapturing ? await audio.stopAudioCapture() : await audio.startAudioCapture();

		logs.logAction('Audio Toggle', result, 'audio');
		showNotification(result.message, result.success ? 'success' : 'error');
	};

	const handleDeviceChange = async (deviceIndex: number): Promise<void> => {
		selectedDeviceIndex.value = deviceIndex;
		// Logique pour changer de device audio si nécessaire
		logs.log(`Audio device changed to index ${deviceIndex}`, 'info', 'audio');
	};

	const handleGainChange = async (gain: number): Promise<void> => {
		const result = await audio.setAudioGain(gain);
		if (!result.success) {
			logs.logAction('Gain Change', result, 'audio');
		}
	};

	// Handlers pour Colors
	const handleColorModeChange = async (mode: string): Promise<void> => {
		const result = await colors.setColorMode(mode);
		logs.logAction('Color Mode Change', result, 'effects');
		showNotification(result.message, result.success ? 'success' : 'error');
	};

	const handleCustomColorChange = async (color: CustomColor): Promise<void> => {
		await colors.setCustomColor(color);
	};

	const handleApplyCustomColor = async (): Promise<void> => {
		const result = await colors.setCustomColor();
		logs.logAction('Apply Custom Color', result, 'effects');
		showNotification(result.message, result.success ? 'success' : 'error');
	};

	// Handlers pour LED
	const handleLEDToggle = async (): Promise<void> => {
		const result = led.isRunning ? await led.stopLEDOutput() : await led.startLEDOutput();

		logs.logAction('LED Toggle', result, 'led');
		showNotification(result.message, result.success ? 'success' : 'error');
	};

	const handleBrightnessChange = async (brightness: number): Promise<void> => {
		const result = await led.setLEDBrightness(brightness);
		if (!result.success) {
			logs.logAction('Brightness Change', result, 'led');
		}
	};

	const handleLEDModeChange = async (mode: string): Promise<void> => {
		// Redémarrer avec le nouveau mode
		if (led.isRunning) {
			await led.stopLEDOutput();
			const result = await led.startLEDOutput(mode as 'simulator' | 'production');
			logs.logAction('LED Mode Change', result, 'led');
			showNotification(result.message, result.success ? 'success' : 'error');
		}
	};

	// Handlers pour Presets
	const handleApplyPreset = async (presetId: string): Promise<void> => {
		const result = await presets.applyPreset(presetId, {
			audio,
			effects,
			colors,
			led,
		});
		logs.logAction('Apply Preset', result, 'user');
		showNotification(result.message, result.success ? 'success' : 'error');
	};

	const handleCreatePreset = async (name: string, description: string): Promise<void> => {
		const config = await presets.captureCurrentConfig({
			audio,
			effects,
			colors,
			led,
		});

		const result = await presets.createPreset(name, description, config);
		logs.logAction('Create Preset', result, 'user');
		showNotification(result.message, result.success ? 'success' : 'error');
	};

	const handleDuplicatePreset = async (presetId: string): Promise<void> => {
		const preset = presets.getPresetById(presetId);
		if (preset) {
			const result = await presets.createPreset(`${preset.name} (Copy)`, preset.description, preset.config);
			logs.logAction('Duplicate Preset', result, 'user');
			showNotification(result.message, result.success ? 'success' : 'error');
		}
	};

	const handleDeletePreset = (presetId: string): void => {
		const result = presets.deletePreset(presetId);
		logs.logAction('Delete Preset', result, 'user');
		showNotification(result.message, result.success ? 'success' : 'error');
	};

	const handleExportPresets = (): void => {
		const result = presets.exportPresets();
		logs.logAction('Export Presets', result, 'user');
		showNotification(result.message, result.success ? 'success' : 'error');
	};

	const handleImportPresets = async (file: File): Promise<void> => {
		const result = await presets.importPresets(file);
		logs.logAction('Import Presets', result, 'user');
		showNotification(result.message, result.success ? 'success' : 'error');
	};

	// Handlers pour System
	const handleQuickStart = async (): Promise<void> => {
		const result = await system.quickStart();
		logs.logAction('Quick Start', result, 'system');
		showNotification(result.message, result.success ? 'success' : 'error');
	};

	const handleShutdown = async (): Promise<void> => {
		const result = await system.shutdown();
		logs.logAction('Shutdown', result, 'system');
		showNotification(result.message, result.success ? 'success' : 'error');
	};

	const handleHealthCheck = async (): Promise<void> => {
		const result = await system.healthCheck();
		logs.logAction('Health Check', result, 'system');
	};

	const handleRefreshStats = async (): Promise<void> => {
		const result = await system.getSystemStats();
		logs.logAction('Refresh Stats', result, 'system');
	};

	const handleStreamToggle = async (): Promise<void> => {
		const isCurrentlyStreaming = audio.state.isCapturing && led.isRunning;

		if (isCurrentlyStreaming) {
			await handleAudioToggle();
			await handleLEDToggle();
		} else {
			await handleAudioToggle();
			await handleLEDToggle();
		}
	};

	// Utilities
	const showNotification = (message: string, type: string): void => {
		notification.value = { message, type };
		setTimeout(() => {
			notification.value = null;
		}, 3000);
	};

	// Lifecycle
	onMounted(async () => {
		logs.initLogs();
		logs.log('🚀 DJ4LED Application started', 'success', 'system');

		// Initialiser les presets depuis le storage
		presets.initializeFromStorage();

		// Démarrer le monitoring système
		system.startMonitoring(5000);

		// Charger les données initiales
		await Promise.all([
			audio.getAudioDevices(),
			effects.getAvailableEffects(),
			led.getLEDControllers(),
			colors.getColorMode(),
		]);
	});

	onUnmounted(() => {
		system.stopMonitoring();
		logs.log('👋 DJ4LED Application stopped', 'info', 'system');
	});
</script>

<style scoped>
	.app-container {
		min-height: 100vh;
		background: #0d1117;
		color: #c9d1d9;
		font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
	}

	/* Navigation */
	.main-nav {
		background: #161b22;
		border-bottom: 1px solid #30363d;
		position: sticky;
		top: 0;
		z-index: 50;
	}

	.nav-container {
		display: flex;
		padding: 0 1rem;
		gap: 0.5rem;
		overflow-x: auto;
	}

	.nav-tab {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 1rem 1.5rem;
		border: none;
		background: transparent;
		color: #8b949e;
		cursor: pointer;
		transition: all 0.2s ease;
		border-bottom: 2px solid transparent;
		white-space: nowrap;
		position: relative;
	}

	.nav-tab:hover {
		color: #c9d1d9;
		background: rgba(200, 209, 217, 0.05);
	}

	.nav-tab.active {
		color: #c9d1d9;
		border-bottom-color: #c9d1d9;
		background: rgba(200, 209, 217, 0.1);
	}

	.tab-icon {
		font-size: 1.2rem;
	}

	.tab-label {
		font-weight: 500;
	}

	.tab-badge {
		background: #c9d1d9;
		color: #0d1117;
		font-size: 0.75rem;
		font-weight: 600;
		padding: 0.125rem 0.375rem;
		border-radius: 10px;
		min-width: 16px;
		text-align: center;
	}

	/* Main Content */
	.main-content {
		padding: 1.5rem;
	}

	.content-grid {
		display: grid;
		grid-template-columns: 1fr 350px;
		gap: 1.5rem;
		max-width: 1400px;
		margin: 0 auto;
	}

	.main-column {
		min-width: 0;
	}

	.tab-content {
		animation: fadeIn 0.3s ease-in-out;
	}

	@keyframes fadeIn {
		from {
			opacity: 0;
			transform: translateY(10px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}

	/* Sidebar */
	.sidebar {
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
		min-width: 0;
	}

	.console-container {
		flex: 1;
		min-height: 300px;
	}

	/* Debug toggle */
	.debug-toggle {
		position: fixed;
		bottom: 1rem;
		right: 1rem;
		width: 48px;
		height: 48px;
		border-radius: 50%;
		border: 1px solid #30363d;
		background: #161b22;
		color: #8b949e;
		cursor: pointer;
		font-size: 1.5rem;
		transition: all 0.2s ease;
		z-index: 100;
	}

	.debug-toggle:hover {
		background: #21262d;
		color: #c9d1d9;
	}

	.debug-toggle.active {
		background: #c9d1d9;
		color: #0d1117;
		border-color: #c9d1d9;
	}

	/* Notifications */
	.notification {
		position: fixed;
		top: 1rem;
		right: 1rem;
		padding: 1rem 1.5rem;
		border-radius: 8px;
		color: white;
		font-weight: 500;
		z-index: 1000;
		animation: slideIn 0.3s ease-out;
		max-width: 400px;
		word-wrap: break-word;
	}

	.notification.success {
		background: #2ea043;
		border: 1px solid #2ea043;
	}

	.notification.error {
		background: #f85149;
		border: 1px solid #f85149;
	}

	.notification.warning {
		background: #d29922;
		border: 1px solid #d29922;
	}

	@keyframes slideIn {
		from {
			transform: translateX(100%);
			opacity: 0;
		}
		to {
			transform: translateX(0);
			opacity: 1;
		}
	}

	/* Responsive */
	@media (max-width: 1024px) {
		.content-grid {
			grid-template-columns: 1fr;
			grid-template-rows: 1fr auto;
		}

		.sidebar {
			grid-row: 2;
		}

		.console-container {
			min-height: 250px;
		}
	}

	@media (max-width: 768px) {
		.main-content {
			padding: 1rem;
		}

		.nav-container {
			padding: 0 0.5rem;
		}

		.nav-tab {
			padding: 0.75rem 1rem;
		}

		.tab-label {
			display: none;
		}

		.debug-toggle {
			bottom: 0.5rem;
			right: 0.5rem;
			width: 40px;
			height: 40px;
			font-size: 1.25rem;
		}

		.notification {
			top: 0.5rem;
			right: 0.5rem;
			left: 0.5rem;
			max-width: none;
		}
	}

	/* Scrollbar global */
	:deep(*::-webkit-scrollbar) {
		width: 8px;
		height: 8px;
	}

	:deep(*::-webkit-scrollbar-track) {
		background: #161b22;
	}

	:deep(*::-webkit-scrollbar-thumb) {
		background: #30363d;
		border-radius: 4px;
	}

	:deep(*::-webkit-scrollbar-thumb:hover) {
		background: #484f58;
	}

	/* Focus styles pour l'accessibilité */
	.nav-tab:focus {
		outline: 2px solid #58a6ff;
		outline-offset: -2px;
	}

	.debug-toggle:focus {
		outline: 2px solid #58a6ff;
		outline-offset: 2px;
	}
</style>
