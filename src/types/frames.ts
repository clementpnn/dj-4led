export interface FrameData {
	width: number;
	height: number;
	format: string;
	data_size: number;
	data: number[];
	timestamp: number;
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
