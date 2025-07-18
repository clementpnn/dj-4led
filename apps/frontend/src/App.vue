<!-- src/App.vue -->
<template>
    <div class="app">
        <Header :isConnected="connection.isConnected.value" :fps="streaming.fps.value" />

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
                @stream="handleStreamToggle"
            />

            <!-- LED Preview Section - Toujours visible -->
            <LedPreview
                :frame-data="streaming.frameData.value"
                :spectrum-data="streaming.spectrumData.value"
                :fps="streaming.fps.value"
                :is-streaming="streaming.isStreaming.value"
                :is-connected="connection.isConnected.value"
                :width="64"
                :height="64"
            />

            <!-- Stream Controls -->
            <div class="stream-controls" v-if="connection.isConnected.value">
                <button
                    @click="handleStreamToggle"
                    class="stream-btn"
                    :class="{ active: streaming.isStreaming.value }"
                    :disabled="streaming.loading.value"
                >
                    {{ streaming.isStreaming.value ? 'Stop Stream' : 'Start Stream' }}
                </button>

                <div class="stream-info" v-if="streaming.isStreaming.value">
                    <span class="info-badge">{{ streaming.streamStats.value.frames }} frames</span>
                    <span class="info-badge">{{ streaming.streamStats.value.spectrum }} spectrum</span>
                    <span class="info-badge">{{ streaming.streamStats.value.packets }} packets</span>
                    <span class="health-indicator" :class="{ healthy: streaming.isStreamHealthy() }">
                        {{ streaming.isStreamHealthy() ? 'Healthy' : 'Unhealthy' }}
                    </span>
                </div>
            </div>

            <!-- Error Display -->
            <div v-if="streaming.error.value" class="error-display">⚠️ {{ streaming.error.value }}</div>

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
            <Terminal :logs="logs.logs.value" @clear-logs="logs.clearLogs" ref="terminalRef" />
        </div>
    </div>
</template>

<script setup lang="ts">
import { onMounted, ref, watch } from 'vue';

// Components from monorepo/ui
import {
    ColorModesPanel,
    CustomColorPanel,
    EffectsPanel,
    Header,
    LedPreview,
    QuickActions,
    Terminal,
} from '@monorepo/ui';

// Composables
import { useColors } from './composables/useColors';
import { useConnection } from './composables/useConnection';
import { useEffects } from './composables/useEffects';
import { useLogs } from './composables/useLogs';
import { useStreaming } from './composables/useStreaming';

// Constants
import { COLOR_CHANNELS, COLOR_MODES, EFFECTS } from './utils/constants';

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
    logs.log(result.message, result.success ? 'success' : 'error');
};

const handleDisconnect = async (): Promise<void> => {
    // Stop streaming first
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
    logs.log('🏓 Sending ping...', 'info');
    const result = await connection.ping();
    logs.log(result.message, result.success ? 'success' : 'warning');
};

// Streaming handlers
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

// Effects handlers
const handleEffectChange = async (effectId: number): Promise<void> => {
    logs.log(`🎇 Applying effect ${effectId}...`, 'info');
    const result = await effects.setEffect(effectId);
    logs.log(result.message, result.success ? 'success' : 'error');
};

// Color handlers
const handleModeChange = async (mode: string): Promise<void> => {
    logs.log(`🌈 Applying mode ${mode}...`, 'info');
    const result = await colors.setColorMode(mode);
    logs.log(result.message, result.success ? 'success' : 'error');
};

const handleColorApply = async (): Promise<void> => {
    const { r, g, b } = colors.customColor.value;
    logs.log(`🎨 Applying RGB(${r.toFixed(2)}, ${g.toFixed(2)}, ${b.toFixed(2)})...`, 'info');
    const result = await colors.setCustomColor();
    logs.log(result.message, result.success ? 'success' : 'error');
};

const handleColorUpdate = (newColor: { r: number; g: number; b: number }): void => {
    colors.customColor.value = newColor;
};

// Watch for log container changes to enable auto-scroll
watch(
    () => logs.logs.value.length,
    () => {
        if (terminalRef.value?.logContainer) {
            logs.logContainer.value = terminalRef.value.logContainer;
        }
    }
);

// Watch for stream health
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

// Watch for FPS changes (optional logging)
watch(
    () => streaming.fps.value,
    (newFps, oldFps) => {
        if (streaming.isStreaming.value && newFps !== oldFps && newFps > 0) {
            // Log FPS changes every 10 FPS or significant drops
            if (Math.abs(newFps - (oldFps || 0)) >= 10 || (oldFps && newFps < oldFps * 0.8)) {
                logs.log(`📊 FPS: ${newFps}`, 'info');
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

/* Main content */
.main-content {
    max-width: 1200px;
    margin: 0 auto;
    padding: 2rem;
}

/* Stream controls */
.stream-controls {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    margin-bottom: 2rem;
    flex-wrap: wrap;
}

.stream-btn {
    background: linear-gradient(45deg, #238636, #2ea043);
    color: white;
    padding: 0.75rem 2rem;
    border: none;
    border-radius: 50px;
    font-weight: 600;
    font-size: 1rem;
    cursor: pointer;
    transition: all 0.3s ease;
    position: relative;
    overflow: hidden;
}

.stream-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
}

.stream-btn::before {
    content: '';
    position: absolute;
    top: 0;
    left: -100%;
    width: 100%;
    height: 100%;
    background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.2), transparent);
    transition: left 0.5s;
}

.stream-btn:hover:not(:disabled)::before {
    left: 100%;
}

.stream-btn:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow: 0 8px 24px rgba(35, 134, 54, 0.4);
}

.stream-btn.active {
    background: linear-gradient(45deg, #da3633, #f85149);
}

.stream-btn.active:hover:not(:disabled) {
    box-shadow: 0 8px 24px rgba(218, 54, 51, 0.4);
}

.stream-info {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
    align-items: center;
}

.info-badge {
    background: rgba(33, 38, 45, 0.8);
    color: #7d8590;
    padding: 0.25rem 0.75rem;
    border-radius: 20px;
    font-size: 0.75rem;
    font-weight: 500;
    border: 1px solid #30363d;
    backdrop-filter: blur(4px);
}

.health-indicator {
    background: linear-gradient(45deg, #da3633, #f85149);
    color: white;
    padding: 0.25rem 0.75rem;
    border-radius: 20px;
    font-size: 0.75rem;
    font-weight: 600;
    transition: all 0.3s ease;
    text-transform: uppercase;
    letter-spacing: 0.5px;
}

.health-indicator.healthy {
    background: linear-gradient(45deg, #238636, #2ea043);
}

/* Error display */
.error-display {
    background: rgba(248, 81, 73, 0.1);
    border: 1px solid #f85149;
    color: #f85149;
    padding: 1rem;
    border-radius: 12px;
    margin-bottom: 2rem;
    font-weight: 500;
    text-align: center;
    backdrop-filter: blur(4px);
}

/* Control grid */
.control-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(350px, 1fr));
    gap: 1.5rem;
    margin-bottom: 2rem;
}

/* Loading state */
.stream-btn:disabled {
    background: linear-gradient(45deg, #656d76, #8b949e);
    cursor: not-allowed;
}

.stream-btn:disabled::before {
    display: none;
}

/* Responsive */
@media (max-width: 768px) {
    .main-content {
        padding: 1rem;
    }

    .control-grid {
        grid-template-columns: 1fr;
    }

    .stream-controls {
        flex-direction: column;
    }

    .stream-info {
        justify-content: center;
    }
}

/* Dark mode enhancements */
@media (prefers-color-scheme: dark) {
    .preview-container {
        box-shadow: 0 4px 32px rgba(0, 0, 0, 0.3);
    }
}

/* Accessibility */
@media (prefers-reduced-motion: reduce) {
    .preview-container:hover {
        transform: none;
    }

    .stream-btn:hover {
        transform: none;
    }

    .stream-btn::before {
        display: none;
    }
}

/* Focus styles */
.stream-btn:focus {
    outline: 2px solid #58a6ff;
    outline-offset: 2px;
}
</style>
