<template>
	<div id="app" class="app-container">
		<!-- Header avec status global -->
		<Header
			:is-connected="isConnected"
			:fps="framesStore.stats.fps"
			:is-streaming="audioStore.state.isCapturing && ledStore.isRunning"
			:stream-loading="systemStore.loading"
			:is-stream-healthy="systemStore.isHealthy"
			:loading="systemStore.loading"
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
						:spectrum-data="audioStore.state.spectrum"
						:fps="framesStore.stats.fps"
						:is-streaming="audioStore.state.isCapturing && ledStore.isRunning"
						:is-connected="isConnected"
						:audio-data="{
							isCapturing: audioStore.state.isCapturing,
							gain: audioStore.state.currentGain,
							deviceCount: audioStore.state.devices.length,
							peak: calculateSpectrumPeak(audioStore.state.spectrum),
							rms: calculateSpectrumRMS(audioStore.state.spectrum),
						}"
						:led-data="{
							isRunning: ledStore.isRunning,
							mode: ledStore.currentMode,
							brightness: ledStore.brightness,
							controllers: ledStore.controllerCount,
							frameSize: ledStore.frameSize?.toString() || 'N/A',
						}"
						:effects-data="{
							current: effectsStore.currentEffect?.name || 'none',
							available: effectsStore.availableEffects.length,
							colorMode: colorsStore.currentMode,
							transitioning: effectsStore.currentEffect?.transitioning || false,
						}"
						:system-data="{
							health: systemStore.health.status,
							healthScore: systemStore.health.score,
							monitoring: systemStore.loading,
						}"
					/>
				</aside>
			</div>
		</main>
	</div>
</template>

<script setup lang="ts">
	import { computed, onMounted, onUnmounted, ref } from 'vue';

	import AudioPanel from '@/components/AudioPanel.vue';
	import ControlPanel from '@/components/ControlPanel.vue';
	import DataPanel from '@/components/DataPanel.vue';
	import Header from '@/components/Header.vue';
	import LEDPanel from '@/components/LEDPanel.vue';
	import Navigation from '@/components/Navigation.vue';
	import PresetsPanel from '@/components/PresetsPanel.vue';
	import SystemPanel from '@/components/SystemPanel.vue';
	import Terminal from '@/components/Terminal.vue';
	import { useAudio } from '@/composables/useAudio';
	import { useColors } from '@/composables/useColors';
	import { useEffects } from '@/composables/useEffects';
	import { useFrames } from '@/composables/useFrames';
	import { useLED } from '@/composables/useLED';
	import { useLogs } from '@/composables/useLogs';
	import { useSystem } from '@/composables/useSystem';
	import { useAudioStore } from '@/stores/audio';
	import { useColorsStore } from '@/stores/colors';
	import { useEffectsStore } from '@/stores/effects';
	import { useFramesStore } from '@/stores/frames';
	import { useLEDStore } from '@/stores/led';
	import { usePresetsStore } from '@/stores/presets';
	import { useSystemStore } from '@/stores/system';

	// State
	const activeTab = ref('audio');
	const isConnected = ref(false);
	const connectionQuality = ref(100);
	const pingMs = ref(0);

	// Stores - pour les données réactives
	const audioStore = useAudioStore();
	const colorsStore = useColorsStore();
	const effectsStore = useEffectsStore();
	const framesStore = useFramesStore();
	const ledStore = useLEDStore();
	const presetsStore = usePresetsStore();
	const systemStore = useSystemStore();

	// Composables - pour les actions uniquement
	const audioComposable = useAudio();
	const colorsComposable = useColors();
	const effectsComposable = useEffects();
	const framesComposable = useFrames();
	const ledComposable = useLED();
	const logsComposable = useLogs();
	const systemComposable = useSystem();

	// Helper functions pour les calculs audio
	const calculateSpectrumPeak = (spectrum: readonly number[]): number => {
		if (!spectrum || spectrum.length === 0) return 0;
		return Math.max(...Array.from(spectrum));
	};

	const calculateSpectrumRMS = (spectrum: readonly number[]): number => {
		if (!spectrum || spectrum.length === 0) return 0;
		const sum = Array.from(spectrum).reduce((acc, val) => acc + val * val, 0);
		return Math.sqrt(sum / spectrum.length);
	};

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
			if (logsComposable.logInfo) {
				logsComposable.logInfo('Starting quick start sequence...', 'system');
			}

			// Simuler la connexion
			isConnected.value = true;
			connectionQuality.value = 100;

			// Démarrer l'audio en premier
			if (!audioStore.state.isCapturing && audioComposable.startCapture) {
				const audioResult = await audioComposable.startCapture();
				if (logsComposable.logAction) {
					logsComposable.logAction('Audio Start', audioResult, 'audio');
				}
			}

			// Démarrer LED
			if (!ledStore.isRunning && ledComposable.startOutput) {
				const ledResult = await ledComposable.startOutput();
				if (logsComposable.logAction) {
					logsComposable.logAction('LED Start', ledResult, 'led');
				}
			}

			if (logsComposable.logSuccess) {
				logsComposable.logSuccess('Quick start completed successfully', 'system');
			}
		} catch (error) {
			const message = error instanceof Error ? error.message : 'Quick start failed';
			if (logsComposable.logError) {
				logsComposable.logError(`Quick start failed: ${message}`, 'system');
			}
		}
	};

	const handleShutdown = async (): Promise<void> => {
		try {
			if (logsComposable.logInfo) {
				logsComposable.logInfo('Shutting down system...', 'system');
			}

			// Arrêter LED
			if (ledStore.isRunning && ledComposable.stopOutput) {
				const ledResult = await ledComposable.stopOutput();
				if (logsComposable.logAction) {
					logsComposable.logAction('LED Stop', ledResult, 'led');
				}
			}

			// Arrêter audio
			if (audioStore.state.isCapturing && audioComposable.stopCapture) {
				const audioResult = await audioComposable.stopCapture();
				if (logsComposable.logAction) {
					logsComposable.logAction('Audio Stop', audioResult, 'audio');
				}
			}

			// Simuler la déconnexion
			isConnected.value = false;
			connectionQuality.value = 0;

			if (logsComposable.logSuccess) {
				logsComposable.logSuccess('System shutdown completed', 'system');
			}
		} catch (error) {
			const message = error instanceof Error ? error.message : 'Shutdown failed';
			if (logsComposable.logError) {
				logsComposable.logError(`Shutdown failed: ${message}`, 'system');
			}
		}
	};

	const handleHealthCheck = async (): Promise<void> => {
		try {
			const startTime = Date.now();

			// Vérifier le status de tous les composants
			const healthPromises = [];

			if (audioComposable.getStatus) healthPromises.push(audioComposable.getStatus());
			if (effectsComposable.getCurrentEffect) healthPromises.push(effectsComposable.getCurrentEffect());
			if (ledComposable.getStatus) healthPromises.push(ledComposable.getStatus());
			if (framesComposable.getCurrentFrame) healthPromises.push(framesComposable.getCurrentFrame());

			const results = await Promise.allSettled(healthPromises);

			pingMs.value = Date.now() - startTime;

			const failures = results.filter((r) => r.status === 'rejected').length;
			const message =
				failures === 0
					? `Health check passed (${pingMs.value}ms)`
					: `Health check completed with ${failures} issues (${pingMs.value}ms)`;

			if (logsComposable.logAction) {
				logsComposable.logAction('Health Check', { success: failures === 0, message }, 'system');
			}
		} catch (error) {
			if (logsComposable.logError) {
				logsComposable.logError('Health check failed', 'system');
			}
			pingMs.value = 0;
		}
	};

	const handleStreamToggle = async (): Promise<void> => {
		const isCurrentlyStreaming = audioStore.state.isCapturing && ledStore.isRunning;

		try {
			if (isCurrentlyStreaming) {
				if (logsComposable.logInfo) {
					logsComposable.logInfo('Stopping audio-visual stream...', 'system');
				}
				await handleShutdown();
			} else {
				if (logsComposable.logInfo) {
					logsComposable.logInfo('Starting audio-visual stream...', 'system');
				}
				await handleQuickStart();
			}
		} catch (error) {
			const message = error instanceof Error ? error.message : 'Stream toggle failed';
			if (logsComposable.logError) {
				logsComposable.logError(`Stream toggle error: ${message}`, 'system');
			}
		}
	};

	// Lifecycle
	onMounted(async () => {
		if (logsComposable.logInfo) {
			logsComposable.logInfo('🚀 DJ4LED Application started', 'system');
		}

		try {
			// Initialiser les composables (seulement ceux qui ont une méthode initialize)
			const initPromises = [];

			if (audioComposable.initialize) initPromises.push(audioComposable.initialize());
			if (effectsComposable.initialize) initPromises.push(effectsComposable.initialize());
			if (colorsComposable.initialize) initPromises.push(colorsComposable.initialize());
			if (ledComposable.initialize) initPromises.push(ledComposable.initialize());
			if (framesComposable.initialize) initPromises.push(framesComposable.initialize());
			if (systemComposable.initialize) initPromises.push(systemComposable.initialize());

			await Promise.allSettled(initPromises);

			// Les presets - accès via le store
			const presetsCount = presetsStore.presets?.length || 0;

			if (logsComposable.logInfo) {
				logsComposable.logInfo(`Presets ready: ${presetsCount} custom + defaults available`, 'system');
			}

			// Auto-switch vers system si problèmes critiques
			if (systemStore.health.status === 'critical') {
				activeTab.value = 'system';
			}

			if (logsComposable.logSuccess) {
				logsComposable.logSuccess('Application initialized successfully', 'system');
			}
		} catch (error) {
			const message = error instanceof Error ? error.message : 'Initialization failed';
			if (logsComposable.logError) {
				logsComposable.logError(`Initialization failed: ${message}`, 'system');
			}
			activeTab.value = 'system';
		}
	});

	onUnmounted(() => {
		if (logsComposable.logInfo) {
			logsComposable.logInfo('👋 DJ4LED Application stopped', 'system');
		}

		// Cleanup des composables - seulement ceux qui ont la méthode cleanup
		const composablesWithCleanup = [
			{ composable: audioComposable, name: 'audio' },
			{ composable: effectsComposable, name: 'effects' },
			{ composable: colorsComposable, name: 'colors' },
			{ composable: ledComposable, name: 'led' },
			{ composable: framesComposable, name: 'frames' },
			{ composable: systemComposable, name: 'system' },
		];

		composablesWithCleanup.forEach(({ composable, name }) => {
			if (composable && 'cleanup' in composable && typeof composable.cleanup === 'function') {
				try {
					composable.cleanup();
				} catch (error) {
					console.warn(`Error during ${name} composable cleanup:`, error);
				}
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
