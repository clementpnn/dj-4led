<template>
  <div class="led-display-container">
    <h3>🌈 LED Display</h3>
    <div class="led-info">
      <span>{{ width }}x{{ height }} LEDs</span>
      <span v-if="fps > 0">{{ fps }} FPS</span>
    </div>
    <canvas
      ref="canvas"
      :width="width * pixelSize"
      :height="height * pixelSize"
      class="led-canvas"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";
import { listen } from "@tauri-apps/api/event";

interface Props {
  width?: number;
  height?: number;
  pixelSize?: number;
}

const props = withDefaults(defineProps<Props>(), {
  width: 128,
  height: 128,
  pixelSize: 4,
});

const canvas = ref<HTMLCanvasElement | null>(null);
const fps = ref(0);
let ctx: CanvasRenderingContext2D | null = null;
let frameCount = 0;
let lastFpsUpdate = Date.now();
let unlistenFrame: (() => void) | null = null;
let unlistenCompressed: (() => void) | null = null;

const drawFrame = (data: Uint8Array) => {
  if (!ctx || !canvas.value) return;

  // Clear canvas with black background
  ctx.fillStyle = "#000000";
  ctx.fillRect(0, 0, canvas.value.width, canvas.value.height);

  // Expected frame size
  const expectedSize = props.width * props.height * 3;
  if (data.length !== expectedSize) {
    console.warn(`Invalid frame size: ${data.length} (expected ${expectedSize})`);
    return;
  }

  // Draw each LED pixel
  for (let y = 0; y < props.height; y++) {
    for (let x = 0; x < props.width; x++) {
      const idx = (y * props.width + x) * 3;
      const r = data[idx];
      const g = data[idx + 1];
      const b = data[idx + 2];

      ctx.fillStyle = `rgb(${r}, ${g}, ${b})`;
      ctx.fillRect(
        x * props.pixelSize,
        y * props.pixelSize,
        props.pixelSize,
        props.pixelSize
      );
    }
  }

  // Update FPS
  frameCount++;
  const now = Date.now();
  if (now - lastFpsUpdate > 1000) {
    fps.value = Math.round((frameCount * 1000) / (now - lastFpsUpdate));
    frameCount = 0;
    lastFpsUpdate = now;
  }
};

const decompressFrame = async (compressedData: Uint8Array): Promise<Uint8Array> => {
  try {
    // Use the Compression Streams API if available
    if ('DecompressionStream' in window) {
      const ds = new DecompressionStream('gzip');
      const blob = new Blob([compressedData]);
      const stream = blob.stream().pipeThrough(ds);
      const response = new Response(stream);
      const arrayBuffer = await response.arrayBuffer();
      return new Uint8Array(arrayBuffer);
    } else {
      console.warn('DecompressionStream not available, skipping compressed frame');
      return new Uint8Array(0);
    }
  } catch (error) {
    console.error('Failed to decompress frame:', error);
    return new Uint8Array(0);
  }
};

onMounted(async () => {
  if (canvas.value) {
    ctx = canvas.value.getContext("2d");
    if (ctx) {
      // Set image smoothing off for crisp pixels
      ctx.imageSmoothingEnabled = false;
    }
  }

  // Listen for frame data from Tauri
  unlistenFrame = await listen<Uint8Array>("frame_data", (event) => {
    drawFrame(event.payload);
  });

  // Listen for compressed frame data
  unlistenCompressed = await listen<Uint8Array>("frame_data_compressed", async (event) => {
    const decompressed = await decompressFrame(event.payload);
    if (decompressed.length > 0) {
      drawFrame(decompressed);
    }
  });
});

onUnmounted(() => {
  if (unlistenFrame) unlistenFrame();
  if (unlistenCompressed) unlistenCompressed();
});

// Redraw canvas when dimensions change
watch([() => props.width, () => props.height, () => props.pixelSize], () => {
  if (canvas.value && ctx) {
    canvas.value.width = props.width * props.pixelSize;
    canvas.value.height = props.height * props.pixelSize;
    ctx.imageSmoothingEnabled = false;
  }
});
</script>

<style scoped>
.led-display-container {
  background: rgba(0, 0, 0, 0.9);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 10px;
  padding: 20px;
  margin: 20px 0;
}

.led-display-container h3 {
  color: #fff;
  margin: 0 0 15px 0;
  font-size: 20px;
  text-align: center;
}

.led-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 15px;
  color: rgba(255, 255, 255, 0.7);
  font-size: 14px;
}

.led-info span {
  background: rgba(255, 255, 255, 0.1);
  padding: 4px 12px;
  border-radius: 4px;
}

.led-canvas {
  display: block;
  margin: 0 auto;
  border: 2px solid rgba(255, 255, 255, 0.2);
  border-radius: 4px;
  max-width: 100%;
  height: auto;
  image-rendering: pixelated;
  image-rendering: -moz-crisp-edges;
  image-rendering: crisp-edges;
}

/* Responsive sizing */
@media (max-width: 768px) {
  .led-display-container {
    padding: 15px;
  }

  .led-canvas {
    max-width: 100%;
  }
}
</style>
