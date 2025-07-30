export interface FrameData {
	width: number;
	height: number;
	format: string;
	data_size: number;
	data: readonly number[];
	timestamp: number;
	statistics?: {
		average_brightness: number;
		max_brightness: number;
		active_pixels: number;
		total_pixels: number;
		activity_percentage: number;
	};
}

export interface FrameStats {
	fps: number;
	frameCount: number;
	droppedFrames: number;
	averageFrameTime: number;
	lastFrameTime: number;
}

export interface FrameMetrics {
	totalFrames: number;
	successRate: number;
	averageFPS: number;
	peakFPS: number;
	minFPS: number;
}
