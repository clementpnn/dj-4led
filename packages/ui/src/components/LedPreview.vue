<!-- src/components/LedPreview.vue -->
<template>
    <div class="led-preview">
        <!-- Header -->
        <div class="preview-header">
            <div class="matrix-info">
                <h2 class="matrix-title">LED Matrix Preview</h2>
                <div class="matrix-specs">
                    <span v-if="frameData" class="spec">{{ getFormatName(frameData.format) }}</span>
                    <span v-if="frameData" class="spec">{{ frameData.width }}x{{ frameData.height }}</span>
                </div>
            </div>
            <div class="status-area">
                <div :class="['status', getStatusClass()]">
                    <div class="status-dot"></div>
                    <span>{{ getStatusText() }}</span>
                </div>
                <div v-if="displayFps > 0" class="fps">{{ displayFps }}fps</div>
            </div>
        </div>

        <!-- Canvas LED Matrix -->
        <div class="canvas-container" :class="{ streaming: isStreaming, offline: !isConnected }">
            <canvas ref="canvasRef" class="led-canvas" @click="handleCanvasClick" />
        </div>

        <!-- Audio Spectrum -->
        <div class="spectrum-section">
            <div class="spectrum-header">
                <h2 class="spectrum-title">Audio Spectrum</h2>
                <div class="spectrum-info">
                    <span>Peak: {{ maxSpectrumValue.toFixed(2) }}</span>
                    <span>Updates: {{ spectrumCount }}</span>
                </div>
            </div>

            <div class="spectrum-container">
                <div class="spectrum-bars">
                    <div
                        v-for="(value, index) in processedSpectrumData"
                        :key="index"
                        class="spectrum-bar"
                        :style="getSpectrumBarStyle(value, index)"
                    />
                </div>
            </div>
        </div>

        <!-- Debug (optionnel) -->
        <div v-if="debugMode" class="debug-section">
            <div class="debug-header">
                <h3>Debug Info</h3>
                <button @click="clearDebugLogs" class="clear-btn">Clear</button>
            </div>
            <div class="debug-logs">
                <div v-for="(log, index) in debugLogs" :key="index" class="debug-log" :class="log.type">
                    <span class="debug-time">{{ log.time }}</span>
                    <span class="debug-message">{{ log.message }}</span>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import type { FrameData } from '../types';

interface Props {
    width?: number;
    height?: number;
    frameData?: FrameData | null;
    spectrumData?: number[];
    fps?: number;
    isStreaming?: boolean;
    isConnected?: boolean;
    debugMode?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
    width: 64,
    height: 64,
    frameData: null,
    spectrumData: () => [],
    fps: 0,
    isStreaming: false,
    isConnected: false,
    debugMode: false,
});

// Refs
const canvasRef = ref<HTMLCanvasElement | null>(null);
const frameCount = ref(0);
const spectrumCount = ref(0);
const debugLogs = ref<Array<{ time: string; message: string; type: string }>>([]);

// Canvas state
let ctx: CanvasRenderingContext2D | null = null;
let currentWidth = props.width;
let currentHeight = props.height;

// Computed
const displayFps = computed(() => props.fps || 0);

const processedSpectrumData = computed(() => {
    if (!props.spectrumData || props.spectrumData.length === 0) {
        return Array(32).fill(0); // 32 barres par défaut
    }
    return props.spectrumData.slice(0, 64); // Limiter à 64 barres max
});

const maxSpectrumValue = computed(() => {
    if (!props.spectrumData || props.spectrumData.length === 0) return 0;
    return Math.max(...props.spectrumData);
});

// Debug
const addDebugLog = (message: string, type: 'info' | 'success' | 'warning' | 'error' = 'info') => {
    if (!props.debugMode) return;

    const time = new Date().toLocaleTimeString();
    debugLogs.value.push({ time, message, type });

    if (debugLogs.value.length > 30) {
        debugLogs.value.shift();
    }

    console.log(`[LedPreview] ${message}`);
};

const clearDebugLogs = () => {
    debugLogs.value = [];
};

// Status methods
const getStatusClass = (): string => {
    if (!props.isConnected) return 'offline';
    if (!props.isStreaming) return 'ready';
    return 'live';
};

const getStatusText = (): string => {
    if (!props.isConnected) return 'OFFLINE';
    if (!props.isStreaming) return 'READY';
    return 'LIVE';
};

const getFormatName = (format: number): string => {
    const formats: Record<number, string> = {
        1: 'RGB',
        2: 'RGBA',
        3: 'HSV',
        4: 'HSL',
    };
    return formats[format] || `Format${format}`;
};

// Spectrum styling
const getSpectrumBarStyle = (value: number, index: number) => {
    const normalizedValue = Math.max(0, Math.min(1, value));
    const height = Math.max(2, normalizedValue * 100);
    const hue = 200 - (index / processedSpectrumData.value.length) * 60; // Bleu vers vert
    const saturation = 70 + normalizedValue * 30;
    const lightness = 40 + normalizedValue * 40;

    return {
        height: `${height}%`,
        backgroundColor: `hsl(${hue}, ${saturation}%, ${lightness}%)`,
        opacity: 0.7 + normalizedValue * 0.3,
    };
};

// Canvas methods
const initCanvas = () => {
    const canvas = canvasRef.value;
    if (!canvas) {
        addDebugLog('Canvas not found', 'error');
        return;
    }

    // Définir la taille du canvas
    canvas.width = currentWidth;
    canvas.height = currentHeight;

    // Obtenir le contexte
    ctx = canvas.getContext('2d', { alpha: false });
    if (!ctx) {
        addDebugLog('Failed to get canvas context', 'error');
        return;
    }

    // Configuration du contexte
    ctx.imageSmoothingEnabled = false;

    // Effacer le canvas
    clearCanvas();

    addDebugLog(`Canvas initialized: ${currentWidth}x${currentHeight}`, 'success');
};

const clearCanvas = () => {
    if (!ctx) return;

    ctx.fillStyle = '#000000';
    ctx.fillRect(0, 0, currentWidth, currentHeight);
};

const renderFrame = (frameData: FrameData) => {
    if (!ctx || !frameData) {
        addDebugLog('No context or frame data', 'error');
        return;
    }

    // Vérifier si les dimensions ont changé
    if (frameData.width !== currentWidth || frameData.height !== currentHeight) {
        currentWidth = frameData.width;
        currentHeight = frameData.height;
        addDebugLog(`Canvas resized: ${currentWidth}x${currentHeight}`, 'info');
        nextTick(() => initCanvas());
        return;
    }

    const { width, height, format, data } = frameData;

    addDebugLog(`Rendering frame: ${width}x${height}, format: ${format}, data: ${data.length}b`, 'info');

    try {
        // Créer ImageData
        const imageData = ctx.createImageData(width, height);
        const pixelCount = width * height;

        if (format === 1) {
            // RGB Format
            if (data.length !== pixelCount * 3) {
                addDebugLog(`RGB data size mismatch: expected ${pixelCount * 3}, got ${data.length}`, 'error');
                return;
            }

            for (let i = 0; i < pixelCount; i++) {
                const srcIndex = i * 3;
                const dstIndex = i * 4;

                imageData.data[dstIndex] = data[srcIndex] || 0; // R
                imageData.data[dstIndex + 1] = data[srcIndex + 1] || 0; // G
                imageData.data[dstIndex + 2] = data[srcIndex + 2] || 0; // B
                imageData.data[dstIndex + 3] = 255; // A
            }
        } else if (format === 2) {
            // RGBA Format
            if (data.length !== pixelCount * 4) {
                addDebugLog(`RGBA data size mismatch: expected ${pixelCount * 4}, got ${data.length}`, 'error');
                return;
            }

            for (let i = 0; i < data.length; i++) {
                imageData.data[i] = data[i] || 0;
            }
        } else {
            addDebugLog(`Unsupported format: ${format}`, 'error');
            return;
        }

        // Dessiner sur le canvas
        ctx.putImageData(imageData, 0, 0);
        frameCount.value++;

        addDebugLog(`Frame rendered successfully (${frameCount.value})`, 'success');
    } catch (error) {
        addDebugLog(`Error rendering frame: ${error}`, 'error');
    }
};

const handleCanvasClick = (event: MouseEvent) => {
    if (!canvasRef.value) return;

    const rect = canvasRef.value.getBoundingClientRect();
    const x = Math.floor(((event.clientX - rect.left) / rect.width) * currentWidth);
    const y = Math.floor(((event.clientY - rect.top) / rect.height) * currentHeight);

    addDebugLog(`Canvas clicked: (${x}, ${y})`, 'info');
};

// Watchers
watch(
    () => props.frameData,
    newFrameData => {
        if (newFrameData) {
            renderFrame(newFrameData);
        } else {
            clearCanvas();
        }
    },
    { immediate: true }
);

watch(
    () => props.spectrumData,
    newSpectrumData => {
        if (newSpectrumData && newSpectrumData.length > 0) {
            spectrumCount.value++;
        }
    }
);

watch(
    () => props.isConnected,
    isConnected => {
        if (!isConnected) {
            clearCanvas();
            frameCount.value = 0;
            spectrumCount.value = 0;
        }
    }
);

// Lifecycle
onMounted(async () => {
    addDebugLog('LedPreview mounted', 'info');
    await nextTick();
    initCanvas();
});

onUnmounted(() => {
    addDebugLog('LedPreview unmounted', 'info');
});
</script>

<style scoped>
.led-preview {
    background: rgba(22, 27, 34, 0.95);
    border: 1px solid #30363d;
    border-radius: 12px;
    padding: 1.5rem;
    color: #f0f6fc;
    margin-bottom: 1.5rem;
}

/* Header */
.preview-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
}

.matrix-title {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
}

.matrix-specs {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.5rem;
}

.spec {
    background: rgba(33, 38, 45, 0.8);
    color: #7d8590;
    padding: 0.25rem 0.75rem;
    border-radius: 20px;
    font-size: 0.75rem;
    border: 1px solid #30363d;
}

.status-area {
    display: flex;
    align-items: center;
    gap: 0.75rem;
}

.status {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    border-radius: 20px;
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
}

.status.offline {
    background: rgba(248, 81, 73, 0.1);
    color: #f85149;
    border: 1px solid #f85149;
}

.status.ready {
    background: rgba(88, 166, 255, 0.1);
    color: #58a6ff;
    border: 1px solid #58a6ff;
}

.status.live {
    background: rgba(35, 134, 54, 0.2);
    color: #2ea043;
    border: 1px solid #2ea043;
}

.status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: currentColor;
    animation: pulse 1.5s infinite;
}

@keyframes pulse {
    0%,
    100% {
        opacity: 1;
    }
    50% {
        opacity: 0.5;
    }
}

.fps {
    background: rgba(35, 134, 54, 0.2);
    color: #2ea043;
    padding: 0.25rem 0.75rem;
    border-radius: 12px;
    font-size: 0.75rem;
    font-weight: 600;
    border: 1px solid #2ea043;
}

/* Canvas */
.canvas-container {
    display: flex;
    justify-content: center;
    margin-bottom: 2rem;
    border-radius: 8px;
    background: #000;
    border: 2px solid #30363d;
    padding: 1rem;
}

.canvas-container.streaming {
    border-color: #2ea043;
    box-shadow: 0 0 20px rgba(46, 160, 67, 0.3);
}

.canvas-container.offline {
    border-color: #f85149;
    box-shadow: 0 0 20px rgba(248, 81, 73, 0.2);
}

.led-canvas {
    width: 320px;
    height: 320px;
    image-rendering: pixelated;
    image-rendering: -moz-crisp-edges;
    image-rendering: crisp-edges;
    cursor: crosshair;
    background: #000;
}

/* Spectrum */
.spectrum-section {
    margin-bottom: 1.5rem;
}

.spectrum-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
}

.spectrum-title {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
}

.spectrum-info {
    display: flex;
    gap: 1rem;
    color: #7d8590;
    font-size: 0.75rem;
    font-family: monospace;
}

.spectrum-container {
    background: #000;
    border: 1px solid #30363d;
    border-radius: 8px;
    padding: 1rem;
    height: 120px;
}

.spectrum-bars {
    display: flex;
    align-items: flex-end;
    height: 100%;
    gap: 1px;
}

.spectrum-bar {
    flex: 1;
    min-height: 2px;
    transition: all 0.1s ease;
    border-radius: 1px 1px 0 0;
}

/* Debug */
.debug-section {
    background: rgba(13, 17, 23, 0.8);
    border: 1px solid #30363d;
    border-radius: 8px;
    padding: 1rem;
}

.debug-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
}

.debug-header h3 {
    margin: 0;
    font-size: 1rem;
}

.clear-btn {
    background: rgba(248, 81, 73, 0.1);
    color: #f85149;
    border: 1px solid #f85149;
    padding: 0.25rem 0.75rem;
    border-radius: 6px;
    font-size: 0.75rem;
    cursor: pointer;
}

.clear-btn:hover {
    background: rgba(248, 81, 73, 0.2);
}

.debug-logs {
    max-height: 200px;
    overflow-y: auto;
    font-family: monospace;
    font-size: 0.75rem;
}

.debug-log {
    display: flex;
    gap: 0.5rem;
    padding: 0.25rem 0;
    border-bottom: 1px solid rgba(48, 54, 61, 0.2);
}

.debug-log:last-child {
    border-bottom: none;
}

.debug-log.success {
    color: #2ea043;
}
.debug-log.warning {
    color: #d29922;
}
.debug-log.error {
    color: #f85149;
}
.debug-log.info {
    color: #7d8590;
}

.debug-time {
    min-width: 80px;
    opacity: 0.7;
}

/* Responsive */
@media (max-width: 768px) {
    .preview-header {
        flex-direction: column;
        gap: 1rem;
    }

    .led-canvas {
        width: 280px;
        height: 280px;
    }
}

@media (max-width: 480px) {
    .led-canvas {
        width: 240px;
        height: 240px;
    }
}
</style>
