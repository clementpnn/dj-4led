<template>
    <div class="app">
        <Header
            :isConnected="connection.isConnected.value"
            :fps="streaming.fps.value"
            :isStreaming="streaming.isStreaming.value"
            :streamLoading="streaming.loading.value"
            :isStreamHealthy="streaming.isStreamHealthy()"
            :loading="connection.loading.value"
            :pingMs="connection.pingMs.value"
            @connect="handleConnect"
            @disconnect="handleDisconnect"
            @ping="handlePing"
            @stream-toggle="handleStreamToggle"
        />

        <!-- Main content -->
        <div class="main-content">
            <!-- Data Panel Section - Always visible with enhanced spectrum -->
            <PanelData
                :spectrum-data="streaming.spectrumData.value"
                :fps="streaming.fps.value"
                :is-streaming="streaming.isStreaming.value"
                :is-connected="connection.isConnected.value"
                :debug-mode="false"
            />

            <!-- Error Display -->
            <div v-if="streaming.error.value" class="error-display">
                <div class="error-icon">⚠️</div>
                <div class="error-content">
                    <div class="error-title">Stream Error</div>
                    <div class="error-message">{{ streaming.error.value }}</div>
                </div>
                <button @click="clearError" class="error-dismiss">✕</button>
            </div>

        <ColorModesPanel
          :color-modes="COLOR_MODES"
          :current-mode="colors.currentMode.value"
          :is-connected="connection.isConnected.value"
          :loading="colors.loading.value"
          @mode-change="handleModeChange"
        />

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

      <Terminal
        ref="terminalRef"
        :logs="logs.logs.value"
        @clear-logs="logs.clearLogs"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref, watch } from "vue";

import { ColorModesPanel, CustomColorPanel, EffectsPanel, Header, PanelData, Terminal } from '@monorepo/ui';

import { useColors } from "./composables/useColors";
import { useConnection } from "./composables/useConnection";
import { useEffects } from "./composables/useEffects";
import { useLogs } from "./composables/useLogs";
import { useStreaming } from "./composables/useStreaming";

import { COLOR_CHANNELS, COLOR_MODES, EFFECTS } from "./utils/constants";

const connection = useConnection();
const effects = useEffects();
const colors = useColors();
const streaming = useStreaming();
const logs = useLogs();

const terminalRef = ref<InstanceType<typeof TerminalComponent> | undefined>(
  undefined,
);
const clearError = () => {
    streaming.clearError();
};

const handleConnect = async (): Promise<void> => {
  const result = await connection.connect();
  logs.log(result.message, result.success ? "success" : "error");
};

const handleDisconnect = async (): Promise<void> => {
    if (streaming.isStreaming.value) {
        await streaming.stopStream();
        logs.log('🔴 Stream stopped', 'info');
    }

    const result = await connection.disconnect();
    // Reset all states when disconnecting
    effects.resetEffect();
    colors.resetColors();
    streaming.reset();
    logs.log(result.message, result.success ? 'success' : 'warning');
};

const handlePing = async (): Promise<void> => {
  logs.log("🏓 Sending ping...", "info");
  const result = await connection.ping();
  logs.log(result.message, result.success ? "success" : "warning");
};

const handleStreamToggle = async (): Promise<void> => {
    if (!connection.isConnected.value) {
        logs.log('❌ Please connect to server first', 'error');
        return;
    }

    if (streaming.isStreaming.value) {
        logs.log('🔴 Stopping stream...', 'info');
        const result = await streaming.stopStream();
        logs.log(result.message, result.success ? 'success' : 'error');
    } else {
        logs.log('🟢 Starting stream...', 'info');
        const result = await streaming.startStream();
        logs.log(result.message, result.success ? 'success' : 'error');
    }
};

const handleEffectChange = async (effectId: number): Promise<void> => {
  logs.log(`🎇 Applying effect ${effectId}...`, "info");
  const result = await effects.setEffect(effectId);
  logs.log(result.message, result.success ? "success" : "error");
};

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

watch(
  () => logs.logs.value.length,
  () => {
    if (terminalRef.value?.logContainer) {
      logs.logContainer.value = terminalRef.value.logContainer;
    }
  },
);

watch(
    () => streaming.isStreaming.value,
    isStreaming => {
        if (isStreaming) {
            // Check stream health every 5 seconds
            const healthCheck = setInterval(() => {
                if (!streaming.isStreaming.value) {
                    clearInterval(healthCheck);
                    return;
                }

                if (!streaming.isStreamHealthy()) {
                    logs.log('⚠️ Stream appears unhealthy - no recent data', 'warning');
                }
            }, 5000);
        }
    }
);

// Watch for streaming errors
watch(
    () => streaming.error.value,
    error => {
        if (error) {
            logs.log(`❌ Stream error: ${error}`, 'error');
        }
    }
);

// Watch for FPS changes with better logging
watch(
    () => streaming.fps.value,
    (newFps, oldFps) => {
        if (streaming.isStreaming.value && newFps !== oldFps && newFps > 0) {
            // Log significant FPS changes
            if (Math.abs(newFps - (oldFps || 0)) >= 10 || (oldFps && newFps < oldFps * 0.7)) {
                const direction = newFps > (oldFps || 0) ? '↗️' : '↘️';
                logs.log(`${direction} FPS: ${newFps} (was ${oldFps || 0})`, 'info');
            }
        }
    }
);

// Initialize
onMounted(() => {
    logs.initLogs();
    // Set the log container reference
    if (terminalRef.value?.logContainer) {
        logs.logContainer.value = terminalRef.value.logContainer;
    }

    // Log initial state
    logs.log('🚀 DJ-4LED Frontend initialized', 'info');

    // Log server info on startup
    streaming
        .getServerInfo()
        .then(info => {
            logs.log(`🌐 Server: ${info}`, 'info');
        })
        .catch(err => {
            logs.log(`⚠️ Could not get server info: ${err}`, 'warning');
        });
});
</script>

<style scoped>
.app {
    min-height: 100vh;
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
    background: linear-gradient(135deg, #0d1117 0%, #161b22 100%);
    color: #f0f6fc;
}

.main-content {
  max-width: 1200px;
  margin: 0 auto;
  padding: 2rem;
}

.error-display {
    display: flex;
    align-items: center;
    gap: 1rem;
    background: rgba(248, 81, 73, 0.1);
    border: 1px solid #f85149;
    border-radius: 12px;
    padding: 1rem 1.5rem;
    margin-bottom: 2rem;
    backdrop-filter: blur(4px);
    animation: slideIn 0.3s ease;
}

@keyframes slideIn {
    from {
        opacity: 0;
        transform: translateY(-10px);
    }
    to {
        opacity: 1;
        transform: translateY(0);
    }
}

.error-icon {
    font-size: 1.5rem;
    color: #f85149;
}

.error-content {
    flex: 1;
}

.error-title {
    font-weight: 600;
    color: #f85149;
    margin-bottom: 0.25rem;
}

.error-message {
    color: #f0f6fc;
    font-size: 0.875rem;
    opacity: 0.9;
}

.error-dismiss {
    background: none;
    border: none;
    color: #7d8590;
    font-size: 1.2rem;
    cursor: pointer;
    padding: 0.25rem;
    border-radius: 4px;
    transition: all 0.2s ease;
}

.error-dismiss:hover {
    color: #f85149;
    background: rgba(248, 81, 73, 0.1);
}

/* Control grid */
.control-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(350px, 1fr));
    gap: 1.5rem;
    margin-bottom: 2rem;
}

@media (max-width: 768px) {
  .main-content {
    padding: 1rem;
  }

  .control-grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 480px) {
    .error-display {
        flex-direction: column;
        text-align: center;
    }
}

/* Dark mode enhancements */
@media (prefers-color-scheme: dark) {
    .error-display {
        box-shadow: 0 4px 32px rgba(0, 0, 0, 0.3);
    }
}

/* Accessibility */
@media (prefers-reduced-motion: reduce) {
    .error-display {
        animation: none;
    }
}

/* Focus styles */
.error-dismiss:focus {
    outline: 2px solid #58a6ff;
    outline-offset: 2px;
}
</style>
