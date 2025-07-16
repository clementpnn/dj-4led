<!-- src/App.vue -->
<template>
    <div class="app">
        <!-- Header -->
        <header class="header">
            <div class="logo">
                <div class="logo-icon">🎵</div>
                <h1>DJ-4LED</h1>
                <div class="logo-icon">💡</div>
            </div>
            <div class="status-badge" :class="{ connected: isConnected }">
                <div class="status-dot"></div>
                {{ isConnected ? 'Connected' : 'Disconnected' }}
            </div>
        </header>

        <!-- Main controls -->
        <div class="main-content">
            <!-- Quick actions -->
            <div class="panel quick-actions">
                <button @click="isConnected ? disconnect() : connect()" class="action-btn primary" :disabled="loading">
                    <span class="btn-icon">{{ loading ? '⏳' : isConnected ? '🔌' : '🔌' }}</span>
                    <span>{{ isConnected ? 'Disconnect' : 'Connect' }}</span>
                </button>
                <button @click="ping" class="action-btn secondary" :disabled="!isConnected || loading">
                    <span class="btn-icon">🏓</span>
                    <span>Ping ({{ pingMs }}ms)</span>
                </button>
                <button @click="listenData" class="action-btn accent" :disabled="loading">
                    <span class="btn-icon">📡</span>
                    <span>Stream ({{ fps }} fps)</span>
                </button>
            </div>

            <!-- Control panels grid -->
            <div class="control-grid">
                <!-- Effects panel -->
                <div class="panel effects-panel">
                    <div class="panel-header">
                        <h2>🎇 Effects</h2>
                        <div class="panel-subtitle">Choose your LED effect</div>
                    </div>
                    <div class="effects-grid">
                        <button
                            v-for="effect in effects"
                            :key="effect.id"
                            @click="setEffect(effect.id)"
                            class="effect-card"
                            :class="{ active: currentEffect === effect.id }"
                            :disabled="!isConnected || loading"
                        >
                            <div class="effect-emoji">{{ effect.emoji }}</div>
                            <div class="effect-name">{{ effect.name }}</div>
                        </button>
                    </div>
                </div>

                <!-- Color modes panel -->
                <div class="panel modes-panel">
                    <div class="panel-header">
                        <h2>🌈 Color Modes</h2>
                        <div class="panel-subtitle">Select color pattern</div>
                    </div>
                    <div class="modes-grid">
                        <button
                            v-for="mode in colorModes"
                            :key="mode.value"
                            @click="setColorMode(mode.value)"
                            class="mode-card"
                            :class="{ active: currentMode === mode.value }"
                            :disabled="!isConnected || loading"
                        >
                            <span class="mode-emoji">{{ mode.emoji }}</span>
                            <span class="mode-label">{{ mode.label }}</span>
                        </button>
                    </div>
                </div>

                <!-- Custom color panel -->
                <div class="panel color-panel">
                    <div class="panel-header">
                        <h2>🎨 Custom Color</h2>
                        <div class="panel-subtitle">Create your own color</div>
                    </div>
                    <div class="color-workspace">
                        <div class="color-preview-large" :style="colorPreviewStyle"></div>
                        <div class="color-sliders">
                            <div class="slider-control" v-for="channel in colorChannels" :key="channel.key">
                                <div class="slider-label">
                                    <span class="color-emoji">{{ channel.emoji }}</span>
                                    <span class="color-name">{{ channel.name }}</span>
                                    <span class="color-value">{{ customColor[channel.key].toFixed(2) }}</span>
                                </div>
                                <div class="slider-wrapper">
                                    <input
                                        v-model.number="customColor[channel.key]"
                                        type="range"
                                        min="0"
                                        max="1"
                                        step="0.01"
                                        :class="['slider', channel.key]"
                                    />
                                </div>
                            </div>
                        </div>
                        <button @click="setCustomColor" class="apply-color-btn" :disabled="!isConnected || loading">
                            ✨ Apply Color
                        </button>
                    </div>
                </div>
            </div>

            <!-- Console terminal -->
            <div class="terminal">
                <div class="terminal-header">
                    <div class="terminal-controls">
                        <div class="terminal-dot red"></div>
                        <div class="terminal-dot yellow"></div>
                        <div class="terminal-dot green"></div>
                    </div>
                    <div class="terminal-title">DJ-4LED Console</div>
                    <button @click="clearLogs" class="clear-btn">Clear</button>
                </div>
                <div class="terminal-body" ref="logContainer">
                    <div v-for="(log, index) in logs" :key="index" :class="['terminal-line', log.type]">
                        <span class="terminal-prompt">$</span>
                        <span class="terminal-time">{{ log.time }}</span>
                        <span class="terminal-message">{{ log.message }}</span>
                    </div>
                    <div class="terminal-cursor">_</div>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { computed, nextTick, ref } from 'vue';

// Types
interface CustomColor {
    r: number;
    g: number;
    b: number;
}

interface ColorChannel {
    key: keyof CustomColor;
    name: string;
    emoji: string;
}

// États
const isConnected = ref(false);
const loading = ref(false);
const currentEffect = ref<number | null>(null);
const currentMode = ref<string | null>(null);
const customColor = ref<CustomColor>({ r: 1.0, g: 0.5, b: 0.0 });
const logs = ref<Array<{ time: string; message: string; type: string }>>([]);
const logContainer = ref<HTMLElement>();
const pingMs = ref(0);
const fps = ref(0);

// Données
const effects = [
    { id: 1, name: 'Pulse', emoji: '💓' },
    { id: 2, name: 'Wave', emoji: '🌊' },
    { id: 3, name: 'Strobe', emoji: '⚡' },
    { id: 4, name: 'Rainbow', emoji: '🌈' },
    { id: 5, name: 'Matrix', emoji: '🔢' },
    { id: 6, name: 'Fire', emoji: '🔥' },
    { id: 7, name: 'Ocean', emoji: '🌊' },
    { id: 8, name: 'Space', emoji: '🌌' },
];

const colorModes = [
    { value: 'rainbow', label: 'Rainbow', emoji: '🌈' },
    { value: 'solid', label: 'Solid', emoji: '🔵' },
    { value: 'pulse', label: 'Pulse', emoji: '💓' },
    { value: 'strobe', label: 'Strobe', emoji: '⚡' },
    { value: 'fade', label: 'Fade', emoji: '🌅' },
];

// Canaux de couleur typés
const colorChannels: ColorChannel[] = [
    { key: 'r', name: 'Red', emoji: '🔴' },
    { key: 'g', name: 'Green', emoji: '🟢' },
    { key: 'b', name: 'Blue', emoji: '🔵' },
];

// Computed
const colorPreviewStyle = computed(() => {
    const r = Math.round(customColor.value.r * 255);
    const g = Math.round(customColor.value.g * 255);
    const b = Math.round(customColor.value.b * 255);
    return {
        backgroundColor: `rgb(${r}, ${g}, ${b})`,
    };
});

// Fonctions
const log = (message: string, type: 'info' | 'success' | 'error' | 'warning' = 'info') => {
    logs.value.push({
        time: new Date().toLocaleTimeString(),
        message,
        type,
    });
    nextTick(() => {
        if (logContainer.value) {
            logContainer.value.scrollTop = logContainer.value.scrollHeight;
        }
    });
};

const connect = async () => {
    loading.value = true;
    try {
        log('🔌 Connecting to DJ-4LED server...', 'info');
        const result = (await invoke('dj_connect')) as string;

        if (result.includes('✅')) {
            isConnected.value = true;
            log(result, 'success');
        } else {
            log(result, 'warning');
        }
    } catch (error) {
        log(`❌ Error: ${error}`, 'error');
    } finally {
        loading.value = false;
    }
};

const disconnect = async () => {
    loading.value = true;
    try {
        log('🔌 Disconnecting from DJ-4LED server...', 'info');
        const result = (await invoke('dj_disconnect')) as string;

        if (result.includes('✅')) {
            isConnected.value = false;
            currentEffect.value = null;
            currentMode.value = null;
            log(result, 'success');
        } else {
            log(result, 'warning');
        }
    } catch (error) {
        log(`❌ Disconnect error: ${error}`, 'error');
    } finally {
        loading.value = false;
    }
};

const ping = async () => {
    loading.value = true;
    try {
        log('🏓 Sending ping...', 'info');
        const result = (await invoke('dj_ping')) as string;
        log(result, result.includes('🏓') ? 'success' : 'warning');
    } catch (error) {
        log(`❌ Ping failed: ${error}`, 'error');
    } finally {
        loading.value = false;
    }
};

const setEffect = async (effectId: number) => {
    loading.value = true;
    try {
        log(`🎇 Applying effect ${effectId}...`, 'info');
        const result = (await invoke('dj_set_effect', { effectId })) as string;
        currentEffect.value = effectId;
        log(result, 'success');
    } catch (error) {
        log(`❌ Effect error: ${error}`, 'error');
    } finally {
        loading.value = false;
    }
};

const setColorMode = async (mode: string) => {
    loading.value = true;
    try {
        log(`🌈 Applying mode ${mode}...`, 'info');
        const result = (await invoke('dj_set_color_mode', { mode })) as string;
        currentMode.value = mode;
        log(result, 'success');
    } catch (error) {
        log(`❌ Mode error: ${error}`, 'error');
    } finally {
        loading.value = false;
    }
};

const setCustomColor = async () => {
    loading.value = true;
    try {
        const { r, g, b } = customColor.value;
        log(`🎨 Applying RGB(${r.toFixed(2)}, ${g.toFixed(2)}, ${b.toFixed(2)})...`, 'info');
        const result = (await invoke('dj_set_custom_color', { r, g, b })) as string;
        log(result, 'success');
    } catch (error) {
        log(`❌ Color error: ${error}`, 'error');
    } finally {
        loading.value = false;
    }
};

const listenData = async () => {
    loading.value = true;
    const startTime = performance.now();
    let frameCount = 0;
    try {
        log('📡 Listening to stream...', 'info');
        const result = (await invoke('dj_listen_data')) as string;
        const endTime = performance.now();
        const duration = (endTime - startTime) / 1000; // en secondes

        // Extraire le nombre de frames du résultat
        const frameMatch = result.match(/(\d+) frames/);
        if (frameMatch) {
            frameCount = parseInt(frameMatch[1]);
            fps.value = Math.round(frameCount / duration);
        } else {
            fps.value = 0;
        }

        log(result, 'success');
    } catch (error) {
        log(`❌ Stream error: ${error}`, 'error');
        fps.value = 0;
    } finally {
        loading.value = false;
    }
};

const clearLogs = () => {
    logs.value = [];
};

// Init
log('🎵 DJ-4LED Controller ready!', 'info');
log('📡 Server: udp://127.0.0.1:8081', 'info');
</script>

<style scoped>
/* Global styles */
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    margin: 0;
    padding: 0;
}

.app {
    min-height: 100vh;
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
    background: #0d1117;
    color: #f0f6fc;
    overflow-x: hidden;
}

/* Header */
.header {
    padding: 2rem;
    text-align: center;
    border-bottom: 1px solid #21262d;
}

.logo {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    margin-bottom: 1.5rem;
}

.logo-icon {
    font-size: 2.5rem;
    opacity: 0.8;
}

.logo h1 {
    font-size: 3rem;
    font-weight: 600;
    margin: 0;
    color: #f0f6fc;
    letter-spacing: 1px;
}

.status-badge {
    display: inline-flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1.5rem;
    background: #21262d;
    border: 1px solid #30363d;
    border-radius: 8px;
    font-weight: 500;
    font-size: 0.9rem;
}

.status-badge.connected {
    background: #0d4929;
    border-color: #1a7f37;
    color: #2ea043;
}

.status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #f85149;
}

.status-badge.connected .status-dot {
    background: #2ea043;
}

/* Main content */
.main-content {
    max-width: 1200px;
    margin: 0 auto;
    padding: 2rem;
}

/* Panels */
.panel {
    background: #161b22;
    border: 1px solid #30363d;
    border-radius: 12px;
    padding: 1.5rem;
    margin-bottom: 1.5rem;
}

.panel:hover {
    border-color: #484f58;
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

/* Quick actions */
.quick-actions {
    display: flex;
    gap: 1rem;
    justify-content: center;
    flex-wrap: wrap;
}

.action-btn {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.875rem 1.5rem;
    border: 1px solid #30363d;
    border-radius: 8px;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
    background: #21262d;
    color: #f0f6fc;
}

.action-btn.primary {
    background: #238636;
    border-color: #2ea043;
    color: white;
}

.action-btn.secondary {
    background: #1f6feb;
    border-color: #388bfd;
    color: white;
}

.action-btn.accent {
    background: #da3633;
    border-color: #f85149;
    color: white;
}

.action-btn:hover:not(:disabled) {
    border-color: #484f58;
    background: #30363d;
}

.action-btn.primary:hover:not(:disabled) {
    background: #2ea043;
}

.action-btn.secondary:hover:not(:disabled) {
    background: #388bfd;
}

.action-btn.accent:hover:not(:disabled) {
    background: #f85149;
}

.action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

.btn-icon {
    font-size: 1rem;
}

/* Control grid */
.control-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(350px, 1fr));
    gap: 1.5rem;
}

/* Effects */
.effects-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(110px, 1fr));
    gap: 0.75rem;
}

.effect-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    padding: 1rem;
    border: 1px solid #30363d;
    border-radius: 8px;
    background: #0d1117;
    cursor: pointer;
    transition: all 0.2s ease;
}

.effect-card:hover:not(:disabled) {
    border-color: #484f58;
    background: #21262d;
}

.effect-card.active {
    background: #0d4929;
    border-color: #2ea043;
}

.effect-card:disabled {
    opacity: 0.4;
    cursor: not-allowed;
}

.effect-emoji {
    font-size: 1.75rem;
}

.effect-name {
    font-size: 0.8rem;
    font-weight: 500;
    color: #f0f6fc;
}

/* Color modes */
.modes-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
}

.mode-card {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    border: 1px solid #30363d;
    border-radius: 8px;
    background: #0d1117;
    cursor: pointer;
    transition: all 0.2s ease;
    flex: 1;
    min-width: 120px;
    font-size: 0.875rem;
}

.mode-card:hover:not(:disabled) {
    border-color: #484f58;
    background: #21262d;
}

.mode-card.active {
    background: #0d4929;
    border-color: #2ea043;
}

.mode-card:disabled {
    opacity: 0.4;
    cursor: not-allowed;
}

.mode-emoji {
    font-size: 1.2rem;
}

.mode-label {
    font-weight: 500;
}

/* Color picker amélioré */
.color-workspace {
    display: grid;
    grid-template-columns: 120px 1fr;
    gap: 2rem;
    align-items: start;
}

.color-preview-large {
    width: 120px;
    height: 120px;
    border-radius: 12px;
    border: 2px solid #30363d;
    position: relative;
    overflow: hidden;
    transition: all 0.3s ease;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.color-preview-large:hover {
    border-color: #484f58;
    transform: scale(1.05);
    box-shadow: 0 8px 20px rgba(0, 0, 0, 0.4);
}

.color-sliders {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
}

.slider-control {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
}

.slider-label {
    display: flex;
    align-items: center;
    gap: 1rem;
    font-weight: 500;
    font-size: 0.9rem;
}

.color-emoji {
    font-size: 1.2rem;
    filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.3));
}

.color-name {
    min-width: 60px;
    font-weight: 600;
}

.color-value {
    margin-left: auto;
    color: #8b949e;
    font-family: 'SF Mono', Consolas, monospace;
    font-size: 0.85rem;
    background: #21262d;
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    border: 1px solid #30363d;
    min-width: 50px;
    text-align: center;
}

.slider-wrapper {
    position: relative;
    padding: 0.25rem 0;
}

.slider {
    width: 100%;
    height: 12px;
    border-radius: 6px;
    outline: none;
    cursor: pointer;
    -webkit-appearance: none;
    appearance: none;
    border: 1px solid #30363d;
    transition: all 0.2s ease;
    position: relative;
    z-index: 2;
}

.slider:hover {
    border-color: #484f58;
}

.slider.r {
    background: linear-gradient(to right, #1a1a1a, #ff4444);
}

.slider.g {
    background: linear-gradient(to right, #1a1a1a, #44ff44);
}

.slider.b {
    background: linear-gradient(to right, #1a1a1a, #4488ff);
}

.slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    background: linear-gradient(135deg, #f0f6fc, #d0d7de);
    border: 3px solid #30363d;
    cursor: pointer;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
    transition: all 0.2s ease;
}

.slider::-webkit-slider-thumb:hover {
    border-color: #484f58;
    transform: scale(1.1);
    box-shadow: 0 6px 16px rgba(0, 0, 0, 0.5);
}

.slider::-webkit-slider-thumb:active {
    transform: scale(0.95);
}

.apply-color-btn {
    grid-column: 1 / -1;
    margin-top: 1.5rem;
    padding: 1rem 2rem;
    border: 1px solid #30363d;
    border-radius: 12px;
    background: linear-gradient(135deg, #238636, #2ea043);
    border-color: #2ea043;
    color: white;
    font-size: 0.9rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.3s ease;
    box-shadow: 0 4px 12px rgba(46, 160, 67, 0.2);
    position: relative;
    overflow: hidden;
}

.apply-color-btn::before {
    content: '';
    position: absolute;
    top: 0;
    left: -100%;
    width: 100%;
    height: 100%;
    background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.2), transparent);
    transition: left 0.5s;
}

.apply-color-btn:hover:not(:disabled) {
    background: linear-gradient(135deg, #2ea043, #34d058);
    transform: translateY(-2px);
    box-shadow: 0 8px 20px rgba(46, 160, 67, 0.3);
}

.apply-color-btn:hover:not(:disabled)::before {
    left: 100%;
}

.apply-color-btn:active {
    transform: translateY(0);
}

.apply-color-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    transform: none;
    box-shadow: none;
}

/* Terminal */
.terminal {
    background: #0d1117;
    border: 1px solid #30363d;
    border-radius: 12px;
    overflow: hidden;
    font-family: 'SF Mono', Consolas, monospace;
}

.terminal-header {
    display: flex;
    align-items: center;
    padding: 1rem 1.5rem;
    background: #161b22;
    border-bottom: 1px solid #30363d;
}

.terminal-controls {
    display: flex;
    gap: 0.5rem;
}

.terminal-dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
}

.terminal-dot.red {
    background: #f85149;
}
.terminal-dot.yellow {
    background: #d29922;
}
.terminal-dot.green {
    background: #2ea043;
}

.terminal-title {
    flex: 1;
    text-align: center;
    color: #f0f6fc;
    font-weight: 500;
    font-size: 0.875rem;
}

.clear-btn {
    background: #21262d;
    border: 1px solid #30363d;
    color: #8b949e;
    cursor: pointer;
    padding: 0.375rem 0.75rem;
    border-radius: 6px;
    font-size: 0.75rem;
    transition: all 0.2s ease;
}

.clear-btn:hover {
    background: #30363d;
    color: #f0f6fc;
}

.terminal-body {
    padding: 1rem 1.5rem;
    max-height: 300px;
    overflow-y: auto;
    font-size: 0.8rem;
    line-height: 1.5;
}

.terminal-line {
    display: flex;
    gap: 0.75rem;
    padding: 0.125rem 0;
    color: #8b949e;
}

.terminal-prompt {
    color: #2ea043;
    font-weight: 600;
}

.terminal-time {
    color: #6e7681;
    min-width: 70px;
}

.terminal-line.success .terminal-message {
    color: #2ea043;
}
.terminal-line.error .terminal-message {
    color: #f85149;
}
.terminal-line.warning .terminal-message {
    color: #d29922;
}
.terminal-line.info .terminal-message {
    color: #58a6ff;
}

.terminal-cursor {
    color: #2ea043;
    margin-top: 0.25rem;
}

/* Responsive */
@media (max-width: 768px) {
    .main-content {
        padding: 1rem;
    }

    .control-grid {
        grid-template-columns: 1fr;
    }

    .color-workspace {
        grid-template-columns: 1fr;
        text-align: center;
        gap: 1rem;
    }

    .quick-actions {
        flex-direction: column;
    }

    .logo h1 {
        font-size: 2.5rem;
    }

    .modes-grid {
        flex-direction: column;
    }

    .mode-card {
        min-width: auto;
    }
}
</style>
