// composables/useFrames.ts
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { computed, onMounted, onUnmounted, ref } from 'vue';

import { useFramesStore } from '@/stores/frames';
import type { ActionResult, FrameData } from '@/types';

export function useFrames() {
	// Store instance
	const framesStore = useFramesStore();

	// Local state
	const autoRefresh = ref(true);
	const error = ref<string | null>(null);
	const refreshInterval = ref<number | null>(null);

	// Event listeners
	let unlistenLedStats: UnlistenFn | null = null;
	let unlistenLedStatus: UnlistenFn | null = null;

	// ===== COMPUTED PROPERTIES =====

	const isReceivingFrames = computed(() => {
		const now = Date.now();
		const lastTime = framesStore.stats.lastFrameTime;
		return lastTime > 0 && now - lastTime < 3000; // 3 seconds tolerance
	});

	const healthStatus = computed(() => {
		if (!isReceivingFrames.value) return 'critical';
		const successRate = framesStore.metrics.successRate;
		if (successRate < 90 || framesStore.stats.fps < 15) return 'warning';
		return 'healthy';
	});

	// ===== FRAME RETRIEVAL =====

	const getCurrentFrame = async (): Promise<ActionResult> => {
		framesStore.setLoading(true);
		error.value = null;

		try {
			const result = await invoke<any>('led_get_frame_data');

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

			return {
				success: true,
				message: 'Frame retrieved successfully',
				data: frameData,
			};
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			error.value = errorMessage;
			console.error('Failed to get current frame:', errorMessage);
			return { success: false, message: errorMessage };
		} finally {
			framesStore.setLoading(false);
		}
	};

	const refreshFrame = async (): Promise<ActionResult> => {
		return await getCurrentFrame();
	};

	// ===== FRAME PROCESSING =====

	const frameToImageUrl = (frame?: FrameData): string => {
		const targetFrame = frame || framesStore.currentFrame;
		if (!targetFrame?.data || targetFrame.data.length === 0) {
			return '';
		}

		try {
			const canvas = document.createElement('canvas');
			canvas.width = targetFrame.width;
			canvas.height = targetFrame.height;
			const ctx = canvas.getContext('2d');

			if (!ctx) return '';

			const imageData = ctx.createImageData(targetFrame.width, targetFrame.height);
			const data = imageData.data;
			const frameDataArray = Array.from(targetFrame.data);

			// Convert RGB to RGBA
			for (let i = 0; i < frameDataArray.length; i += 3) {
				const pixelIndex = (i / 3) * 4;
				if (pixelIndex + 3 < data.length) {
					data[pixelIndex] = frameDataArray[i]; // R
					data[pixelIndex + 1] = frameDataArray[i + 1]; // G
					data[pixelIndex + 2] = frameDataArray[i + 2]; // B
					data[pixelIndex + 3] = 255; // A
				}
			}

			ctx.putImageData(imageData, 0, 0);
			return canvas.toDataURL('image/png');
		} catch (err) {
			console.warn('Error converting frame to image:', err);
			return '';
		}
	};

	const downloadFrame = async (frame?: FrameData, filename?: string): Promise<ActionResult> => {
		const targetFrame = frame || framesStore.currentFrame;
		if (!targetFrame) {
			return { success: false, message: 'No frame available to download' };
		}

		try {
			const dataUrl = frameToImageUrl(targetFrame);
			if (!dataUrl) {
				return { success: false, message: 'Failed to convert frame to downloadable format' };
			}

			const response = await fetch(dataUrl);
			const blob = await response.blob();
			const url = URL.createObjectURL(blob);

			const a = document.createElement('a');
			a.href = url;
			a.download = filename || `led-frame-${targetFrame.timestamp || Date.now()}.png`;
			document.body.appendChild(a);
			a.click();
			document.body.removeChild(a);
			URL.revokeObjectURL(url);

			return { success: true, message: 'Frame downloaded successfully' };
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : String(err);
			return { success: false, message: errorMessage };
		}
	};

	// ===== AUTO-REFRESH =====

	const startAutoRefresh = () => {
		if (refreshInterval.value) return;

		refreshInterval.value = window.setInterval(() => {
			if (autoRefresh.value && isReceivingFrames.value) {
				getCurrentFrame();
			}
		}, 1000); // Refresh every second when auto-refresh is on
	};

	const stopAutoRefresh = () => {
		if (refreshInterval.value) {
			clearInterval(refreshInterval.value);
			refreshInterval.value = null;
		}
	};

	const toggleAutoRefresh = () => {
		autoRefresh.value = !autoRefresh.value;
		if (autoRefresh.value) {
			startAutoRefresh();
		} else {
			stopAutoRefresh();
		}
	};

	// ===== EVENT HANDLERS =====

	const handleLedStats = (event: any) => {
		try {
			if (event.payload) {
				framesStore.updateStats({
					fps: event.payload.fps || 0,
					frameCount: event.payload.frame_count || framesStore.stats.frameCount,
				});

				// Auto-refresh frame data periodically
				if (autoRefresh.value && event.payload.frame_count % 30 === 0) {
					getCurrentFrame();
				}
			}
		} catch (err) {
			console.error('Error processing LED stats:', err);
		}
	};

	const handleLedStatus = (event: any) => {
		try {
			if (event.payload?.status === 'started' || event.payload?.status === 'running') {
				if (autoRefresh.value) {
					getCurrentFrame();
					startAutoRefresh();
				}
			} else if (event.payload?.status === 'stopped') {
				stopAutoRefresh();
			}
		} catch (err) {
			console.error('Error processing LED status:', err);
		}
	};

	// ===== SETUP & CLEANUP =====

	const setupListeners = async () => {
		try {
			unlistenLedStats = await listen('led_stats', handleLedStats);
			unlistenLedStatus = await listen('led_status', handleLedStatus);
		} catch (err) {
			console.error('Failed to setup frame event listeners:', err);
			error.value = 'Failed to setup event listeners';
		}
	};

	const cleanup = () => {
		stopAutoRefresh();

		if (unlistenLedStats) {
			unlistenLedStats();
			unlistenLedStats = null;
		}
		if (unlistenLedStatus) {
			unlistenLedStatus();
			unlistenLedStatus = null;
		}
	};

	const initialize = async () => {
		try {
			await setupListeners();
			if (autoRefresh.value) {
				await getCurrentFrame();
				startAutoRefresh();
			}
		} catch (err) {
			console.error('Failed to initialize frames composable:', err);
			error.value = 'Failed to initialize frames system';
		}
	};

	// ===== LIFECYCLE =====

	onMounted(() => {
		initialize();
	});

	onUnmounted(() => {
		cleanup();
	});

	// ===== PUBLIC API =====

	return {
		// State
		autoRefresh,
		error,

		// Computed
		isReceivingFrames,
		healthStatus,

		// Actions
		getCurrentFrame,
		refreshFrame,
		frameToImageUrl,
		downloadFrame,
		toggleAutoRefresh,

		// Utilities
		initialize,
		cleanup,
	};
}
