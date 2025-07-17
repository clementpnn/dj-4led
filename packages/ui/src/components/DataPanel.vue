<!-- src/components/DataPanel.vue -->
<template>
  <div
    v-if="streamData.frames.length > 0 || streamData.spectrum.length > 0"
    class="panel data-panel"
  >
    <div class="panel-header">
      <h2>📊 Live Data</h2>
      <div class="panel-subtitle">Real-time LED and audio data</div>
    </div>

    <div class="data-grid">
      <!-- Spectrum visualization -->
      <div v-if="streamData.spectrum.length > 0" class="spectrum-display">
        <h3>Audio Spectrum</h3>
        <div class="spectrum-bars">
          <div
            v-for="(value, index) in streamData.spectrum"
            :key="index"
            class="spectrum-bar"
            :style="{ height: `${value * 100}%` }"
          ></div>
        </div>
      </div>

      <!-- Frame info -->
      <div v-if="streamData.lastFrame" class="frame-info">
        <h3>Frame Data</h3>
        <div class="frame-stats">
          <div>
            Resolution: {{ streamData.lastFrame.width }}x{{
              streamData.lastFrame.height
            }}
          </div>
          <div>
            Format: {{ streamData.lastFrame.format === 1 ? "RGB" : "Unknown" }}
          </div>
          <div>Size: {{ streamData.lastFrame.size }} bytes</div>
          <div>Frames: {{ streamData.frames.length }}</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
defineProps({
  streamData: {
    type: Object,
    default: () => ({
      frames: [],
      spectrum: [],
      lastFrame: undefined,
    }),
  },
});
</script>

<style scoped>
.panel {
  background: #161b22;
  border: 1px solid #30363d;
  border-radius: 12px;
  padding: 1.5rem;
  margin-bottom: 1.5rem;
}

.data-panel {
  background: #0d1117;
  border: 1px solid #1f6feb;
}

.panel-header {
  margin-bottom: 1.5rem;
}

.panel-header h2 {
  margin: 0 0 0.5rem 0;
  font-size: 1.25rem;
  font-weight: 600;
  color: #f0f6fc;
}

.panel-subtitle {
  color: #8b949e;
  font-size: 0.875rem;
}

.data-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 2rem;
}

.spectrum-display h3,
.frame-info h3 {
  margin: 0 0 1rem 0;
  color: #58a6ff;
  font-size: 1rem;
}

.spectrum-bars {
  display: flex;
  gap: 2px;
  height: 100px;
  align-items: flex-end;
  background: #21262d;
  padding: 0.5rem;
  border-radius: 6px;
}

.spectrum-bar {
  flex: 1;
  background: linear-gradient(to top, #1f6feb, #58a6ff);
  border-radius: 2px 2px 0 0;
  min-height: 2px;
  transition: height 0.1s ease;
}

.frame-stats {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  font-family: "SF Mono", Consolas, monospace;
  font-size: 0.875rem;
  color: #8b949e;
}

@media (max-width: 768px) {
  .data-grid {
    grid-template-columns: 1fr;
    gap: 1rem;
  }
}
</style>
