import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { computed, onMounted, onUnmounted } from 'vue';
import { useFramesStore } from '../stores/frames';
import { useLogsStore } from '../stores/logs';
import type { ActionResult, FrameData } from '../types';

export function useFrames() {
	const framesStore = useFramesStore();
	const logsStore = useLogsStore();

	let unlistenFrameData: UnlistenFn | null = null;
	let unlistenFrameCompressed: UnlistenFn | null = null;

	// Computed properties pour faciliter l'utilisation
	const isReceivingFrames = computed(() => {
		const now = Date.now();
		const lastFrameTime = framesStore.stats.lastFrameTime;
		return lastFrameTime > 0 && now - lastFrameTime < 5000; // 5 secondes de tolérance
	});

	const frameRate = computed(() => {
		if (framesStore.frameHistory.length < 2) return 0;

		const frames = framesStore.frameHistory.slice(-10); // Dernières 10 frames
		if (frames.length < 2) return 0;

		const timeSpan = frames[frames.length - 1].timestamp - frames[0].timestamp;
		return timeSpan > 0 ? Math.round((frames.length * 1000) / timeSpan) : 0;
	});

	// Récupérer la frame actuelle
	const getCurrentFrame = async (): Promise<ActionResult> => {
		framesStore.setLoading(true);
		try {
			const frame = await invoke<FrameData>('get_current_frame');
			framesStore.setCurrentFrame(frame);
			logsStore.addLog(
				`🖼️ Frame retrieved: ${frame.width}x${frame.height} (${(frame.data_size / 1024).toFixed(1)}KB)`,
				'info',
				'led',
				{ width: frame.width, height: frame.height, size: frame.data_size }
			);
			return { success: true, message: 'Frame retrieved', data: frame };
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			logsStore.addLog(`Failed to get current frame: ${errorMessage}`, 'error', 'led');
			return { success: false, message: `❌ Frame error: ${errorMessage}` };
		} finally {
			framesStore.setLoading(false);
		}
	};

	// Gestionnaire des données de frame
	const handleFrameData = (frame: FrameData): void => {
		// Valider les données de frame
		if (!frame || !frame.data || frame.width <= 0 || frame.height <= 0) {
			logsStore.addLog('Invalid frame data received', 'warning', 'led');
			framesStore.incrementDroppedFrames();
			return;
		}

		// Enrichir la frame avec timestamp si manquant
		if (!frame.timestamp) {
			frame.timestamp = Date.now();
		}

		framesStore.setCurrentFrame(frame);

		// Log périodique (chaque 100ème frame)
		if (framesStore.stats.frameCount % 100 === 0) {
			logsStore.addLog(
				`📊 Frame #${framesStore.stats.frameCount}: ${frame.width}x${frame.height} @${frameRate.value}fps`,
				'debug',
				'led',
				{ frameCount: framesStore.stats.frameCount, fps: frameRate.value }
			);
		}
	};

	// Gestionnaire des frames compressées
	const handleCompressedFrameData = (compressedData: number[]): void => {
		if (!compressedData || compressedData.length === 0) {
			logsStore.addLog('Invalid compressed frame data received', 'warning', 'led');
			framesStore.incrementDroppedFrames();
			return;
		}

		// Créer une frame virtuelle pour les données compressées
		const frame: FrameData = {
			width: 128, // Taille par défaut, pourrait être dynamique
			height: 128,
			format: 'compressed',
			data_size: compressedData.length,
			data: compressedData,
			timestamp: Date.now(),
		};

		framesStore.setCurrentFrame(frame);
		logsStore.addLog(`🗜️ Compressed frame received: ${compressedData.length} bytes`, 'debug', 'led');
	};

	// Configuration des écouteurs d'événements
	const setupEventListeners = async (): Promise<void> => {
		try {
			// Écouter les frames normales
			unlistenFrameData = await listen<FrameData>('frame_data', (event) => {
				try {
					handleFrameData(event.payload);
				} catch (error) {
					logsStore.addLog(`Error processing frame data: ${error}`, 'error', 'led');
					framesStore.incrementDroppedFrames();
				}
			});

			// Écouter les frames compressées
			unlistenFrameCompressed = await listen<number[]>('frame_data_compressed', (event) => {
				try {
					handleCompressedFrameData(event.payload);
				} catch (error) {
					logsStore.addLog(`Error processing compressed frame: ${error}`, 'error', 'led');
					framesStore.incrementDroppedFrames();
				}
			});

			logsStore.addLog('✅ Frame event listeners ready', 'success', 'led');
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			logsStore.addLog(`❌ Error setting up frame event listeners: ${errorMessage}`, 'error', 'led');
		}
	};

	// Nettoyage des écouteurs
	const cleanup = (): void => {
		const listeners = [
			{ ref: unlistenFrameData, name: 'frame_data' },
			{ ref: unlistenFrameCompressed, name: 'frame_data_compressed' },
		];

		listeners.forEach(({ ref, name }) => {
			if (ref) {
				try {
					ref();
					logsStore.addLog(`✅ Cleaned up ${name} listener`, 'debug', 'led');
				} catch (error) {
					logsStore.addLog(`❌ Error cleaning up ${name} listener: ${error}`, 'warning', 'led');
				}
			}
		});

		unlistenFrameData = null;
		unlistenFrameCompressed = null;
	};

	// Convertir les données de frame en URL d'image (pour affichage)
	const frameToImageUrl = (frame: FrameData): string => {
		if (!frame || !frame.data || frame.data.length === 0) {
			return '';
		}

		try {
			const canvas = document.createElement('canvas');
			canvas.width = frame.width;
			canvas.height = frame.height;
			const ctx = canvas.getContext('2d');

			if (!ctx) {
				logsStore.addLog('Failed to get canvas context for frame conversion', 'warning', 'led');
				return '';
			}

			const imageData = ctx.createImageData(frame.width, frame.height);
			const data = imageData.data;

			// Gestion des différents formats
			if (frame.format === 'RGB' || !frame.format) {
				// Convertir RGB en RGBA
				for (let i = 0; i < frame.data.length; i += 3) {
					const pixelIndex = (i / 3) * 4;
					if (pixelIndex + 3 < data.length) {
						data[pixelIndex] = frame.data[i]; // R
						data[pixelIndex + 1] = frame.data[i + 1]; // G
						data[pixelIndex + 2] = frame.data[i + 2]; // B
						data[pixelIndex + 3] = 255; // A
					}
				}
			} else if (frame.format === 'RGBA') {
				// Copier directement les données RGBA
				for (let i = 0; i < Math.min(frame.data.length, data.length); i++) {
					data[i] = frame.data[i];
				}
			}

			ctx.putImageData(imageData, 0, 0);
			return canvas.toDataURL('image/png');
		} catch (error) {
			logsStore.addLog(`Error converting frame to image: ${error}`, 'warning', 'led');
			return '';
		}
	};

	// Convertir une frame en blob pour téléchargement
	const frameToBlob = async (frame: FrameData): Promise<Blob | null> => {
		try {
			const dataUrl = frameToImageUrl(frame);
			if (!dataUrl) return null;

			const response = await fetch(dataUrl);
			return await response.blob();
		} catch (error) {
			logsStore.addLog(`Error converting frame to blob: ${error}`, 'warning', 'led');
			return null;
		}
	};

	// Analyser les métriques de performance des frames
	const analyzePerformance = (): ActionResult => {
		try {
			const metrics = framesStore.metrics;
			const analysis = {
				status: 'good' as 'good' | 'warning' | 'critical',
				issues: [] as string[],
				recommendations: [] as string[],
			};

			// Analyser le taux de réussite
			if (metrics.successRate < 90) {
				analysis.status = 'critical';
				analysis.issues.push(`Low success rate: ${metrics.successRate.toFixed(1)}%`);
				analysis.recommendations.push('Check network connection and LED controllers');
			} else if (metrics.successRate < 95) {
				analysis.status = 'warning';
				analysis.issues.push(`Moderate frame drops: ${metrics.successRate.toFixed(1)}%`);
			}

			// Analyser le FPS
			if (metrics.averageFPS < 15) {
				analysis.status = 'critical';
				analysis.issues.push(`Low FPS: ${metrics.averageFPS.toFixed(1)}`);
				analysis.recommendations.push('Reduce effect complexity or increase LED refresh rate');
			} else if (metrics.averageFPS < 30) {
				if (analysis.status !== 'critical') analysis.status = 'warning';
				analysis.issues.push(`Moderate FPS: ${metrics.averageFPS.toFixed(1)}`);
			}

			logsStore.addLog(
				`📊 Performance analysis: ${analysis.status} (${analysis.issues.length} issues)`,
				analysis.status === 'good' ? 'success' : analysis.status === 'warning' ? 'warning' : 'error',
				'led',
				analysis
			);

			return {
				success: true,
				message: `Performance analysis completed: ${analysis.status}`,
				data: analysis,
			};
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			return { success: false, message: `Performance analysis failed: ${errorMessage}` };
		}
	};

	// Lifecycle
	onMounted(() => {
		logsStore.addLog('🖼️ Frames composable mounted', 'debug', 'led');
		setupEventListeners();
	});

	onUnmounted(() => {
		logsStore.addLog('💀 Frames composable unmounting', 'debug', 'led');
		cleanup();
	});

	return {
		// Store state access
		currentFrame: framesStore.currentFrame,
		frameHistory: framesStore.frameHistory,
		stats: framesStore.stats,
		loading: framesStore.loading,
		hasCurrentFrame: framesStore.hasCurrentFrame,
		frameCount: framesStore.frameCount,
		averageFPS: framesStore.averageFPS,
		metrics: framesStore.metrics,
		recentFrames: framesStore.recentFrames,

		// Computed properties
		isReceivingFrames,
		frameRate,

		// Actions
		getCurrentFrame,
		frameToImageUrl,
		frameToBlob,
		analyzePerformance,
		clearHistory: framesStore.clearHistory,
		cleanup,
		reset: framesStore.reset,
	};
}
