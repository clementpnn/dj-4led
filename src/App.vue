<template>
	<div id="app" class="app-container">
		<!-- Header avec status global -->
		<Header
			:is-connected="isConnected"
			:fps="frames.stats.fps"
			:is-streaming="audio.state.isCapturing && led.isRunning"
			:stream-loading="system.loading"
			:is-stream-healthy="system.isHealthy"
			:loading="system.loading"
			:ping-ms="pingMs"
			@connect="handleQuickStart"
			@disconnect="handleShutdown"
			@ping="handleHealthCheck"
			@stream-toggle="handleStreamToggle"
		/>

		<!-- Navigation principale -->
		<Navigation :active-tab="activeTab" :tabs="tabs" @tab-change="activeTab = $event" />

		<!-- Contenu principal -->
		<main class="main-content">
			<div class="content-layout">
				<!-- Contenu principal (gauche) -->
				<div class="main-column">
					<!-- Audio Panel -->
					<div v-show="activeTab === 'audio'" class="tab-content">
						<AudioPanel />
					</div>

					<!-- Control Panel (Effects + Colors) -->
					<div v-show="activeTab === 'control'" class="tab-content">
						<ControlPanel />
					</div>

					<!-- LED Panel -->
					<div v-show="activeTab === 'led'" class="tab-content">
						<LEDPanel />
					</div>

					<!-- Presets Panel -->
					<div v-show="activeTab === 'presets'" class="tab-content">
						<PresetsPanel />
					</div>

					<!-- System Panel -->
					<div v-show="activeTab === 'system'" class="tab-content">
						<SystemPanel :connection-quality="connectionQuality" :is-online="isConnected" />
					</div>

					<!-- Terminal en bas (toute la largeur) -->
					<div class="terminal-section">
						<Terminal />
					</div>
				</div>

				<!-- Panel de données (droite) -->
				<aside class="data-sidebar">
					<DataPanel
						:spectrum-data="audio.state.spectrum"
						:fps="frames.stats.fps"
						:is-streaming="audio.state.isCapturing && led.isRunning"
						:is-connected="isConnected"
						:audio-data="{
							isCapturing: audio.state.isCapturing,
							gain: audio.state.currentGain,
							deviceCount: audio.state.devices.length,
							peak: audio.spectrumPeak,
							rms: audio.spectrumRMS,
						}"
						:led-data="{
							isRunning: led.isRunning,
							mode: led.currentMode,
							brightness: led.brightness,
							controllers: led.controllerCount,
							frameSize: led.frameSize?.toString() || 'N/A',
						}"
						:effects-data="{
							current: effects.currentEffectName,
							available: effects.availableEffects.length,
							colorMode: colors.currentMode,
							transitioning: effects.isTransitioning,
						}"
						:system-data="{
							health: system.health.status,
							healthScore: system.health.score,
							monitoring: system.loading,
						}"
					/>
				</aside>
			</div>
		</main>
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
	import AudioPanel from './components/AudioPanel.vue';
	import ControlPanel from './components/ControlPanel.vue';
	import DataPanel from './components/DataPanel.vue';
	import Header from './components/Header.vue';
	import LEDPanel from './components/LEDPanel.vue';
	import Navigation from './components/Navigation.vue';
	import PresetsPanel from './components/PresetsPanel.vue';
	import SystemPanel from './components/SystemPanel.vue';
	import Terminal from './components/Terminal.vue';

	// State
	const activeTab = ref('audio');
	const isConnected = ref(false);
	const connectionQuality = ref(100);
	const pingMs = ref(0);

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
		},
		{
			id: 'control',
			label: 'Control',
		},
		{
			id: 'led',
			label: 'LED Output',
		},
		{
			id: 'presets',
			label: 'Presets',
		},
		{
			id: 'system',
			label: 'System',
		},
	]);

	// Handlers
	const handleQuickStart = async (): Promise<void> => {
		try {
			logs.logInfo('Starting quick start sequence...', 'system');

			// Simuler la connexion
			isConnected.value = true;
			connectionQuality.value = 100;

			// Démarrer l'audio en premier
			if (!audio.state.isCapturing) {
				const audioResult = await audio.startCapture();
				logs.logAction('Audio Start', audioResult, 'audio');
			}

			// Démarrer LED
			if (!led.isRunning) {
				const ledResult = await led.startOutput();
				logs.logAction('LED Start', ledResult, 'led');
			}

			logs.logSuccess('Quick start completed successfully', 'system');
		} catch (error) {
			const message = error instanceof Error ? error.message : 'Quick start failed';
			logs.logError(`Quick start failed: ${message}`, 'system');
		}
	};

	const handleShutdown = async (): Promise<void> => {
		try {
			logs.logInfo('Shutting down system...', 'system');

			// Arrêter LED
			if (led.isRunning) {
				const ledResult = await led.stopOutput();
				logs.logAction('LED Stop', ledResult, 'led');
			}

			// Arrêter audio
			if (audio.state.isCapturing) {
				const audioResult = await audio.stopCapture();
				logs.logAction('Audio Stop', audioResult, 'audio');
			}

			// Simuler la déconnexion
			isConnected.value = false;
			connectionQuality.value = 0;

			logs.logSuccess('System shutdown completed', 'system');
		} catch (error) {
			const message = error instanceof Error ? error.message : 'Shutdown failed';
			logs.logError(`Shutdown failed: ${message}`, 'system');
		}
	};

	const handleHealthCheck = async (): Promise<void> => {
		try {
			const startTime = Date.now();

			// Vérifier le status de tous les composants
			const results = await Promise.allSettled([
				audio.getStatus(),
				effects.getCurrentEffect(),
				led.getStatus(),
				frames.getCurrentFrame(),
			]);

			pingMs.value = Date.now() - startTime;

			const failures = results.filter((r) => r.status === 'rejected').length;
			const message =
				failures === 0
					? `Health check passed (${pingMs.value}ms)`
					: `Health check completed with ${failures} issues (${pingMs.value}ms)`;

			logs.logAction('Health Check', { success: failures === 0, message }, 'system');
		} catch (error) {
			logs.logError('Health check failed', 'system');
			pingMs.value = 0;
		}
	};

	const handleStreamToggle = async (): Promise<void> => {
		const isCurrentlyStreaming = audio.state.isCapturing && led.isRunning;

		try {
			if (isCurrentlyStreaming) {
				logs.logInfo('Stopping audio-visual stream...', 'system');
				await handleShutdown();
			} else {
				logs.logInfo('Starting audio-visual stream...', 'system');
				await handleQuickStart();
			}
		} catch (error) {
			const message = error instanceof Error ? error.message : 'Stream toggle failed';
			logs.logError(`Stream toggle error: ${message}`, 'system');
		}
	};

	// Lifecycle
	onMounted(async () => {
		logs.logInfo('🚀 DJ4LED Application started', 'system');

		try {
			// Initialiser les composables
			await Promise.all([
				audio.initialize(),
				effects.initialize(),
				colors.initialize(),
				led.initialize(),
				frames.initialize(),
				system.initialize(),
			]);

			// Les presets se chargent automatiquement via onMounted dans usePresets
			logs.logInfo(`Presets ready: ${presets.presets.length} custom + defaults available`, 'system');

			// Auto-switch vers system si problèmes critiques
			if (system.health.status === 'critical') {
				activeTab.value = 'system';
			}

			logs.logSuccess('Application initialized successfully', 'system');
		} catch (error) {
			const message = error instanceof Error ? error.message : 'Initialization failed';
			logs.logError(`Initialization failed: ${message}`, 'system');
			activeTab.value = 'system';
		}
	});

	onUnmounted(() => {
		logs.logInfo('👋 DJ4LED Application stopped', 'system');

		// Cleanup des composables
		[audio, effects, colors, led, frames, system].forEach((composable) => {
			if (composable.cleanup) {
				composable.cleanup();
			}
		});
	});
</script>

<style scoped>
	.app-container {
		min-height: 100vh;
		background: #0d1117;
		color: #e6edf3;
		font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
	}

	/* Main Content */
	.main-content {
		padding: 1.5rem;
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
	}

	/* Content Layout */
	.content-layout {
		display: grid;
		grid-template-columns: 1fr 350px;
		gap: 1.5rem;
	}

	.main-column {
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
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

	/* Data Sidebar */
	.data-sidebar {
		min-width: 0;
	}

	/* Terminal Section */
	.terminal-section {
		width: 100%;
		min-height: 300px;
	}

	/* Responsive */
	@media (max-width: 1024px) {
		.content-layout {
			grid-template-columns: 1fr;
			grid-template-rows: 1fr auto;
		}

		.data-sidebar {
			grid-row: 2;
		}

		.terminal-section {
			min-height: 250px;
		}
	}

	@media (max-width: 768px) {
		.main-content {
			padding: 1rem;
			gap: 1rem;
		}

		.content-layout {
			gap: 1rem;
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
		background: #21262d;
		border-radius: 4px;
	}

	:deep(*::-webkit-scrollbar-thumb:hover) {
		background: #30363d;
	}
</style>
