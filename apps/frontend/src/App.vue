<!-- src/App.vue -->
<template>
  <div class="app">
    <!-- Header -->
    <Header :is-connected="connection.isConnected.value" />

    <!-- Main content -->
    <div class="main-content">
      <!-- Quick actions -->
      <QuickActions
        :is-connected="connection.isConnected.value"
        :loading="connection.loading.value"
        :ping-ms="connection.pingMs.value"
        :fps="streaming.fps.value"
        @connect="handleConnect"
        @disconnect="handleDisconnect"
        @ping="handlePing"
        @stream="handleStream"
      />

      <!-- Real-time data display -->
      <DataPanel
        v-if="
          streaming.streamData.value.frames.length > 0 ||
          streaming.streamData.value.spectrum.length > 0
        "
        :stream-data="streaming.streamData.value"
      />

      <!-- Control panels grid -->
      <div class="control-grid">
        <!-- Effects panel -->
        <EffectsPanel
          :effects="EFFECTS"
          :current-effect="effects.currentEffect.value"
          :is-connected="connection.isConnected.value"
          :loading="effects.loading.value"
          @effect-change="handleEffectChange"
        />

        <!-- Color modes panel -->
        <ColorModesPanel
          :color-modes="COLOR_MODES"
          :current-mode="colors.currentMode.value"
          :is-connected="connection.isConnected.value"
          :loading="colors.loading.value"
          @mode-change="handleModeChange"
        />

        <!-- Custom color panel -->
        <CustomColorPanel
          :custom-color="colors.customColor.value"
          :color-channels="COLOR_CHANNELS"
          :color-preview-style="colors.colorPreviewStyle.value"
          :is-connected="connection.isConnected.value"
          :loading="colors.loading.value"
          @color-apply="handleColorApply"
          @color-update="handleColorUpdate"
        />
      </div>

      <!-- Console terminal -->
      <Terminal
        :logs="logs.logs.value"
        @clear-logs="logs.clearLogs"
        ref="terminalRef"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref, watch } from "vue";

// Components
import ColorModesPanel from "./components/ColorModesPanel.vue";
import CustomColorPanel from "./components/CustomColorPanel.vue";
import DataPanel from "./components/DataPanel.vue";
import EffectsPanel from "./components/EffectsPanel.vue";
import Header from "./components/Header.vue";
import QuickActions from "./components/QuickActions.vue";
import Terminal from "./components/Terminal.vue";

// Composables
import { useColors } from "./composables/useColors";
import { useConnection } from "./composables/useConnection";
import { useEffects } from "./composables/useEffects";
import { useLogs } from "./composables/useLogs";
import { useStreaming } from "./composables/useStreaming";

// Constants
import { COLOR_CHANNELS, COLOR_MODES, EFFECTS } from "./utils/constants";

// Composables initialization
const connection = useConnection();
const effects = useEffects();
const colors = useColors();
const streaming = useStreaming();
const logs = useLogs();

// Refs
const terminalRef = ref<InstanceType<typeof Terminal> | null>(null);

// Connection handlers
const handleConnect = async (): Promise<void> => {
  const result = await connection.connect();
  logs.log(result.message, result.success ? "success" : "error");
};

const handleDisconnect = async (): Promise<void> => {
  const result = await connection.disconnect();
  // Reset all states when disconnecting
  effects.resetEffect();
  colors.resetColors();
  streaming.clearStreamData();
  logs.log(result.message, result.success ? "success" : "warning");
};

const handlePing = async (): Promise<void> => {
  logs.log("🏓 Sending ping...", "info");
  const result = await connection.ping();
  logs.log(result.message, result.success ? "success" : "warning");
};

const handleStream = async (): Promise<void> => {
  logs.log("📡 Listening to stream...", "info");
  const result = await streaming.listenData();
  logs.log(result.message, result.success ? "success" : "error");
};

// Effects handlers
const handleEffectChange = async (effectId: number): Promise<void> => {
  logs.log(`🎇 Applying effect ${effectId}...`, "info");
  const result = await effects.setEffect(effectId);
  logs.log(result.message, result.success ? "success" : "error");
};

// Color handlers
const handleModeChange = async (mode: string): Promise<void> => {
  logs.log(`🌈 Applying mode ${mode}...`, "info");
  const result = await colors.setColorMode(mode);
  logs.log(result.message, result.success ? "success" : "error");
};

const handleColorApply = async (): Promise<void> => {
  const { r, g, b } = colors.customColor.value;
  logs.log(
    `🎨 Applying RGB(${r.toFixed(2)}, ${g.toFixed(2)}, ${b.toFixed(2)})...`,
    "info",
  );
  const result = await colors.setCustomColor();
  logs.log(result.message, result.success ? "success" : "error");
};

const handleColorUpdate = (newColor: {
  r: number;
  g: number;
  b: number;
}): void => {
  colors.customColor.value = newColor;
};

// Watch for log container changes to enable auto-scroll
watch(
  () => logs.logs.value.length,
  () => {
    if (terminalRef.value?.logContainer) {
      logs.logContainer.value = terminalRef.value.logContainer;
    }
  },
);

// Initialize
onMounted(() => {
  logs.initLogs();
  // Set the log container reference
  if (terminalRef.value?.logContainer) {
    logs.logContainer.value = terminalRef.value.logContainer;
  }
});
</script>

<style scoped>
/* Global styles */

*,
*::before,
*::after {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}
a {
  text-decoration: none;
  color: inherit;
}

ul,
ol {
  list-style: none;
}

button {
  border: none;
  background: none;
  cursor: pointer;
  font-family: inherit;
}

body {
  margin: 0;
  padding: 0;
  background: #0d1117;
}

.app {
  min-height: 100vh;
  font-family:
    "Inter",
    -apple-system,
    BlinkMacSystemFont,
    sans-serif;
  background: #0d1117;
  color: #f0f6fc;
  overflow-x: hidden;
}

/* Main content */
.main-content {
  max-width: 1200px;
  margin: 0 auto;
  padding: 2rem;
}

/* Control grid */
.control-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(350px, 1fr));
  gap: 1.5rem;
}

/* Responsive */
@media (max-width: 768px) {
  .main-content {
    padding: 1rem;
  }

  .control-grid {
    grid-template-columns: 1fr;
  }
}
</style>
