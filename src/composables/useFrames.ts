// composables/useFrames.ts
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue';
import { useFramesStore } from '../stores/frames';
import type { ActionResult, FrameData } from '../types';

export function useFrames() {
	// Store instance
	const framesStore = useFramesStore();

	// Local state for UI-specific functionality
	const frameContainer = ref<HTMLElement | undefined>(undefined);
	const autoRefresh = ref(true);
	const error = ref<string | null>(null);

	// Event listeners references
	let unlistenFrameData: UnlistenFn | null = null;
	let unlistenLedStats: UnlistenFn | null = null;

	// ===== COMPUTED PROPERTIES =====

	const isReceivingFrames = computed(() => {
		const now = Date.now();
		const lastTime = framesStore.stats.lastFrameTime;
		return lastTime > 0 && now - lastTime < 5000; // 5 seconds tolerance
	});

	const currentFPS = computed(() => {
		if (framesStore.frameHistory.length < 2) return 0;

		const recentFrames = framesStore.frameHistory.slice(-10);
		if (recentFrames.length < 2) return 0;

		const timeSpan = recentFrames[recentFrames.length - 1].timestamp - recentFrames[0].timestamp;
		return timeSpan > 0 ? Math.round((recentFrames.length * 1000) / timeSpan) : 0;
	});

	const successRate = computed(() => {
		const total = framesStore.stats.frameCount + framesStore.stats.droppedFrames;
		return total > 0 ? ((framesStore.stats.frameCount - framesStore.stats.droppedFrames) / total) * 100 : 100;
	});

	const healthStatus = computed(() => {
		if (!isReceivingFrames.value) return 'critical';
		if (successRate.value < 90 || currentFPS.value < 15) return 'warning';
		return 'healthy';
	});

	// ===== FRAME RETRIEVAL ACTIONS =====

	const getCurrentFrame = async (): Promise<ActionResult> => {
		framesStore.setLoading(true);
		error.value = null;

		try {
			const result = await invoke<any>('led_get_frame_data');

			// Transform backend data to FrameData format
			const frameData: FrameData = {
				width: result.width,
				height: result.height,
				format: result.format || 'RGB',
				data_size: result.data_size,
				data: Array.from(result.data) as readonly number[],
				timestamp: Date.now(),
				statistics: {
					average_brightness: result.average_brightness || 0,
					max_brightness: Math.max(...result.data),
					active_pixels: result.data.filter((pixel: number) => pixel > 0).length,
					total_pixels: result.data.length,
					activity_percentage:
						(result.data.filter((pixel: number) => pixel > 0).length / result.data.length) * 100,
				},
			};

			framesStore.setCurrentFrame(frameData);

			console.log(
				`🖼️ Frame retrieved: ${frameData.width}x${frameData.height} (${(frameData.data_size / 1024).toFixed(1)}KB)`
			);

			return {
				success: true,
				message: 'Frame retrieved successfully',
				data: frameData,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			error.value = errorMessage;
			console.error('❌ Failed to get current frame:', errorMessage);
			return { success: false, message: errorMessage };
		} finally {
			framesStore.setLoading(false);
		}
	};

	const refreshFrame = async (): Promise<ActionResult> => {
		console.log('🔄 Manual frame refresh requested');
		return await getCurrentFrame();
	};

	// ===== FRAME PROCESSING UTILITIES =====

	const frameToImageUrl = (frame?: FrameData): string => {
		const targetFrame = frame || framesStore.currentFrame;
		if (!targetFrame || !targetFrame.data || targetFrame.data.length === 0) {
			return '';
		}

		try {
			const canvas = document.createElement('canvas');
			canvas.width = targetFrame.width;
			canvas.height = targetFrame.height;
			const ctx = canvas.getContext('2d');

			if (!ctx) {
				console.warn('⚠️ Failed to get canvas context for frame conversion');
				return '';
			}

			const imageData = ctx.createImageData(targetFrame.width, targetFrame.height);
			const data = imageData.data;

			// Convert readonly array to regular array for processing
			const frameDataArray = Array.from(targetFrame.data);

			// Convert RGB to RGBA
			if (targetFrame.format === 'RGB' || !targetFrame.format) {
				for (let i = 0; i < frameDataArray.length; i += 3) {
					const pixelIndex = (i / 3) * 4;
					if (pixelIndex + 3 < data.length) {
						data[pixelIndex] = frameDataArray[i]; // R
						data[pixelIndex + 1] = frameDataArray[i + 1]; // G
						data[pixelIndex + 2] = frameDataArray[i + 2]; // B
						data[pixelIndex + 3] = 255; // A
					}
				}
			} else if (targetFrame.format === 'RGBA') {
				for (let i = 0; i < Math.min(frameDataArray.length, data.length); i++) {
					data[i] = frameDataArray[i];
				}
			}

			ctx.putImageData(imageData, 0, 0);
			return canvas.toDataURL('image/png');
		} catch (err) {
			console.warn('⚠️ Error converting frame to image:', err);
			return '';
		}
	};

	const frameToBlob = async (frame?: FrameData): Promise<Blob | null> => {
		try {
			const dataUrl = frameToImageUrl(frame);
			if (!dataUrl) return null;

			const response = await fetch(dataUrl);
			return await response.blob();
		} catch (err) {
			console.warn('⚠️ Error converting frame to blob:', err);
			return null;
		}
	};

	const downloadFrame = async (frame?: FrameData, filename?: string): Promise<ActionResult> => {
		const targetFrame = frame || framesStore.currentFrame;
		if (!targetFrame) {
			return { success: false, message: 'No frame available to download' };
		}

		try {
			const blob = await frameToBlob(targetFrame);
			if (!blob) {
				return { success: false, message: 'Failed to convert frame to downloadable format' };
			}

			const url = URL.createObjectURL(blob);
			const a = document.createElement('a');
			a.href = url;
			a.download = filename || `frame-${targetFrame.timestamp || Date.now()}.png`;
			document.body.appendChild(a);
			a.click();
			document.body.removeChild(a);
			URL.revokeObjectURL(url);

			console.log('📥 Frame downloaded successfully');
			return { success: true, message: 'Frame downloaded successfully' };
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			console.error('❌ Failed to download frame:', errorMessage);
			return { success: false, message: errorMessage };
		}
	};

	// ===== EXPORT & ANALYSIS =====

	const exportFrameHistory = (): ActionResult => {
		try {
			if (framesStore.frameHistory.length === 0) {
				return { success: false, message: 'No frame history to export' };
			}

			const exportData = {
				timestamp: Date.now(),
				frameCount: framesStore.frameHistory.length,
				stats: framesStore.stats,
				metrics: framesStore.metrics,
				frames: framesStore.frameHistory.map((frame) => ({
					timestamp: frame.timestamp,
					width: frame.width,
					height: frame.height,
					format: frame.format,
					data_size: frame.data_size,
					statistics: frame.statistics,
				})),
			};

			const blob = new Blob([JSON.stringify(exportData, null, 2)], {
				type: 'application/json',
			});
			const url = URL.createObjectURL(blob);
			const a = document.createElement('a');
			a.href = url;
			a.download = `frame-history-${new Date().toISOString().split('T')[0]}.json`;
			document.body.appendChild(a);
			a.click();
			document.body.removeChild(a);
			URL.revokeObjectURL(url);

			console.log('📤 Frame history exported successfully');
			return { success: true, message: 'Frame history exported successfully' };
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			console.error('❌ Failed to export frame history:', errorMessage);
			return { success: false, message: errorMessage };
		}
	};

	const analyzePerformance = (): ActionResult => {
		try {
			const analysis = {
				status: 'healthy' as 'healthy' | 'warning' | 'critical',
				score: 100,
				issues: [] as string[],
				recommendations: [] as string[],
				metrics: framesStore.metrics,
			};

			// Analyze success rate
			if (successRate.value < 80) {
				analysis.status = 'critical';
				analysis.score -= 40;
				analysis.issues.push(`Very low success rate: ${successRate.value.toFixed(1)}%`);
				analysis.recommendations.push('Check network connection and LED controllers');
			} else if (successRate.value < 95) {
				analysis.status = 'warning';
				analysis.score -= 20;
				analysis.issues.push(`Low success rate: ${successRate.value.toFixed(1)}%`);
				analysis.recommendations.push('Monitor network stability');
			}

			// Analyze FPS
			if (currentFPS.value < 10) {
				analysis.status = 'critical';
				analysis.score -= 30;
				analysis.issues.push(`Very low FPS: ${currentFPS.value}`);
				analysis.recommendations.push('Reduce effect complexity or increase refresh rate');
			} else if (currentFPS.value < 30) {
				if (analysis.status !== 'critical') analysis.status = 'warning';
				analysis.score -= 15;
				analysis.issues.push(`Low FPS: ${currentFPS.value}`);
				analysis.recommendations.push('Consider optimizing effects');
			}

			// Check frame reception
			if (!isReceivingFrames.value) {
				analysis.status = 'critical';
				analysis.score -= 50;
				analysis.issues.push('No frames received recently');
				analysis.recommendations.push('Check LED output and audio capture status');
			}

			// Check frame quality
			const currentFrame = framesStore.currentFrame;
			if (currentFrame?.statistics && currentFrame.statistics.average_brightness < 1.0) {
				analysis.score -= 10;
				analysis.issues.push('Very low frame brightness detected');
				analysis.recommendations.push('Check LED brightness settings');
			}

			console.log(`📊 Performance analysis: ${analysis.status} (score: ${analysis.score}/100)`);
			return {
				success: true,
				message: `Performance analysis: ${analysis.status} (${analysis.issues.length} issues)`,
				data: analysis,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			return { success: false, message: `Performance analysis failed: ${errorMessage}` };
		}
	};

	// ===== AUTO-REFRESH FUNCTIONALITY =====

	const handleAutoRefresh = (): void => {
		if (frameContainer.value && autoRefresh.value) {
			nextTick(() => {
				// Trigger any frame display updates here
				// Could be used to update canvas or image displays
			});
		}
	};

	const toggleAutoRefresh = (): void => {
		autoRefresh.value = !autoRefresh.value;
		if (autoRefresh.value) {
			handleAutoRefresh();
		}
		console.log(`🔄 Auto-refresh ${autoRefresh.value ? 'enabled' : 'disabled'}`);
	};

	// ===== EVENT HANDLERS =====

	const handleFrameUpdate = (event: any) => {
		try {
			// Use LED stats as proxy for frame updates since we don't have direct frame events
			if (event.payload && event.payload.frame_count) {
				// Update FPS in store
				framesStore.updateStats({
					fps: event.payload.fps || framesStore.stats.fps,
				});

				// Optionally refresh frame data
				if (autoRefresh.value && event.payload.frame_count % 30 === 0) {
					getCurrentFrame();
				}
			}
		} catch (err) {
			console.error('❌ Error processing frame update:', err);
			framesStore.incrementDroppedFrames();
		}
	};

	// ===== EVENT LISTENERS SETUP =====

	const setupListeners = async (): Promise<void> => {
		try {
			// Listen for LED stats as proxy for frame updates
			unlistenLedStats = await listen('led_stats', handleFrameUpdate);

			console.log('✅ Frame event listeners setup complete');
		} catch (err) {
			console.error('❌ Failed to setup frame event listeners:', err);
			error.value = 'Failed to setup event listeners';
		}
	};

	const cleanup = (): void => {
		const listeners = [
			{ fn: unlistenFrameData, name: 'frame_data' },
			{ fn: unlistenLedStats, name: 'led_stats' },
		];

		listeners.forEach(({ fn, name }) => {
			if (fn) {
				try {
					fn();
					console.log(`✅ Cleaned up ${name} listener`);
				} catch (err) {
					console.warn(`❌ Error cleaning up ${name} listener:`, err);
				}
			}
		});

		unlistenFrameData = null;
		unlistenLedStats = null;
	};

	// ===== INITIALIZATION =====

	const initialize = async (): Promise<void> => {
		console.log('🖼️ Initializing frames composable...');

		try {
			await setupListeners();

			// Get initial frame if auto-refresh is enabled
			if (autoRefresh.value) {
				await getCurrentFrame();
			}

			console.log('✅ Frames composable initialized successfully');
		} catch (err) {
			console.error('❌ Failed to initialize frames composable:', err);
			error.value = 'Failed to initialize frames system';
		}
	};

	// ===== LIFECYCLE =====

	onMounted(() => {
		console.log('🖼️ Frames composable mounted');
		initialize();
	});

	onUnmounted(() => {
		console.log('💀 Frames composable unmounting');
		cleanup();
	});

	// ===== PUBLIC API =====

	return {
		// Store access
		...framesStore,

		// Local state
		frameContainer,
		autoRefresh,
		error,

		// Computed
		isReceivingFrames,
		currentFPS,
		successRate,
		healthStatus,

		// Actions
		getCurrentFrame,
		refreshFrame,
		frameToImageUrl,
		frameToBlob,
		downloadFrame,
		exportFrameHistory,
		analyzePerformance,
		toggleAutoRefresh,

		// Utilities
		initialize,
		cleanup,
	};
}
