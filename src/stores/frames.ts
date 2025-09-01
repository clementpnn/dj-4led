import { defineStore } from 'pinia';
import { computed, readonly, ref } from 'vue';

import { APP_CONFIG } from '@/config';
import type { FrameData, FrameMetrics, FrameStats } from '@/types';

export const useFramesStore = defineStore('frames', () => {
	// ===== STATE =====
	const currentFrame = ref<FrameData | null>(null);
	const frameHistory = ref<FrameData[]>([]);
	const stats = ref<FrameStats>({
		fps: 0,
		frameCount: 0,
		droppedFrames: 0,
		averageFrameTime: 0,
		lastFrameTime: 0,
	});
	const loading = ref(false);

	// ===== GETTERS =====
	const hasCurrentFrame = computed(() => currentFrame.value !== null);
	const frameCount = computed(() => frameHistory.value.length);

	const averageFPS = computed(() => {
		if (frameHistory.value.length < 2) return 0;

		const lastFrame = frameHistory.value[frameHistory.value.length - 1];
		const firstFrame = frameHistory.value[0];

		if (!lastFrame || !firstFrame) return 0;

		const timeSpan = lastFrame.timestamp - firstFrame.timestamp;
		return timeSpan > 0 ? (frameHistory.value.length * 1000) / timeSpan : 0;
	});

	const metrics = computed(
		(): FrameMetrics => ({
			totalFrames: stats.value.frameCount,
			successRate:
				stats.value.frameCount > 0
					? ((stats.value.frameCount - stats.value.droppedFrames) / stats.value.frameCount) * 100
					: 0,
			averageFPS: averageFPS.value,
			peakFPS: Math.max(stats.value.fps, averageFPS.value),
			minFPS: stats.value.fps > 0 ? Math.min(stats.value.fps, averageFPS.value) : 0,
		})
	);

	const recentFrames = computed(() => frameHistory.value.slice(-5));

	// ===== ACTIONS =====
	const setCurrentFrame = (frame: FrameData | null) => {
		currentFrame.value = frame;

		if (frame) {
			// Add to history
			frameHistory.value.push(frame);

			// Limit history size
			if (frameHistory.value.length > APP_CONFIG.performance.frameHistorySize) {
				frameHistory.value = frameHistory.value.slice(-APP_CONFIG.performance.frameHistorySize);
			}

			// Update stats
			stats.value.frameCount++;
			stats.value.lastFrameTime = frame.timestamp;

			// Calculate average frame time
			if (frameHistory.value.length > 1) {
				const timeDiff = frame.timestamp - frameHistory.value[frameHistory.value.length - 2].timestamp;
				stats.value.averageFrameTime = (stats.value.averageFrameTime + timeDiff) / 2;
			}
		}
	};

	const updateStats = (newStats: Partial<FrameStats>) => {
		Object.assign(stats.value, newStats);
	};

	const incrementDroppedFrames = () => {
		stats.value.droppedFrames++;
	};

	const setLoading = (isLoading: boolean) => {
		loading.value = isLoading;
	};

	const clearHistory = () => {
		frameHistory.value = [];
	};

	const reset = () => {
		currentFrame.value = null;
		frameHistory.value = [];
		stats.value = {
			fps: 0,
			frameCount: 0,
			droppedFrames: 0,
			averageFrameTime: 0,
			lastFrameTime: 0,
		};
		loading.value = false;
	};

	return {
		// State
		currentFrame: readonly(currentFrame),
		frameHistory: readonly(frameHistory),
		stats: readonly(stats),
		loading: readonly(loading),

		// Getters
		hasCurrentFrame,
		frameCount,
		averageFPS,
		metrics,
		recentFrames,

		// Actions
		setCurrentFrame,
		updateStats,
		incrementDroppedFrames,
		setLoading,
		clearHistory,
		reset,
	};
});
