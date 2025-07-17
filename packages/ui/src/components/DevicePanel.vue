<!-- src/components/DevicePanel.vue -->
<template>
    <div class="device-panel" :class="deviceStatusClass">
        <!-- Device Header -->
        <div class="device-header">
            <div class="device-info">
                <h3 class="device-name">{{ device.name }}</h3>
                <div class="device-address">{{ device.ipAddress }}:{{ device.port }}</div>
            </div>
            
            <div class="device-status">
                <div class="status-indicator" :class="statusIndicatorClass">
                    {{ statusEmoji }}
                </div>
                <div v-if="device.lastPing" class="ping-info">
                    {{ device.lastPing }}ms
                </div>
            </div>
        </div>

        <!-- Connection Controls -->
        <div class="connection-controls">
            <button 
                class="btn" 
                :class="connectButtonClass"
                @click="handleConnect"
                :disabled="device.isConnecting"
            >
                <span v-if="device.isConnecting">⏳ Connexion...</span>
                <span v-else-if="device.isConnected">🔌 Connecté</span>
                <span v-else>⚡ Connecter</span>
            </button>

            <button 
                class="btn btn-secondary" 
                @click="handleDisconnect"
                :disabled="!device.isConnected || device.isConnecting"
            >
                🔌❌ Déconnecter
            </button>

            <button 
                class="btn btn-info" 
                @click="handlePing"
                :disabled="device.isConnecting"
            >
                🏓 Ping
            </button>
        </div>

        <!-- Quick Controls (only shown when connected) -->
        <div v-if="device.isConnected" class="quick-controls">
            <!-- Effects Quick Select -->
            <div class="control-section">
                <h4>🎇 Effets</h4>
                <div class="effect-buttons">
                    <button 
                        v-for="effect in effects" 
                        :key="effect.id"
                        class="btn btn-small effect-btn"
                        :class="{ active: device.currentEffect === effect.id }"
                        @click="handleEffectChange(effect.id)"
                    >
                        {{ effect.emoji }}
                    </button>
                </div>
            </div>

            <!-- Color Modes Quick Select -->
            <div class="control-section">
                <h4>🌈 Couleurs</h4>
                <div class="color-buttons">
                    <button 
                        v-for="mode in colorModes" 
                        :key="mode.value"
                        class="btn btn-small color-btn"
                        :class="{ active: device.currentColorMode === mode.value }"
                        @click="handleColorModeChange(mode.value)"
                    >
                        {{ mode.emoji }}
                    </button>
                </div>
            </div>

            <!-- Custom Color (simplified) -->
            <div v-if="device.customColor" class="control-section">
                <h4>🎨 Couleur personnalisée</h4>
                <div 
                    class="color-preview" 
                    :style="customColorStyle"
                ></div>
            </div>
        </div>

        <!-- Device Settings -->
        <div class="device-settings">
            <button 
                class="btn btn-danger btn-small"
                @click="handleRemove"
                :disabled="device.isConnected"
            >
                🗑️ Supprimer
            </button>

            <button 
                class="btn btn-secondary btn-small"
                @click="$emit('edit', device.id)"
            >
                ⚙️ Modifier
            </button>
        </div>

        <!-- Last seen info -->
        <div v-if="device.lastSeen" class="device-footer">
            <small class="last-seen">
                Vu: {{ formatLastSeen(device.lastSeen) }}
            </small>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

// Types (to be imported from the main application)
interface Device {
    id: string;
    name: string;
    ipAddress: string;
    port: number;
    isConnected: boolean;
    isConnecting: boolean;
    lastPing?: number;
    lastSeen?: Date;
    currentEffect?: number;
    currentColorMode?: string;
    customColor?: { r: number; g: number; b: number };
}

interface Effect {
    id: number;
    name: string;
    emoji: string;
}

interface ColorMode {
    value: string;
    label: string;
    emoji: string;
}

// Props
interface Props {
    device: Device;
    effects?: Effect[];
    colorModes?: ColorMode[];
}

const props = withDefaults(defineProps<Props>(), {
    effects: () => [],
    colorModes: () => []
});

// Emits
interface Emits {
    connect: [deviceId: string];
    disconnect: [deviceId: string];
    ping: [deviceId: string];
    setEffect: [deviceId: string, effectId: number];
    setColorMode: [deviceId: string, mode: string];
    remove: [deviceId: string];
    edit: [deviceId: string];
}

const emit = defineEmits<Emits>();

// Computed properties
const deviceStatusClass = computed(() => ({
    'device-connected': props.device.isConnected,
    'device-connecting': props.device.isConnecting,
    'device-disconnected': !props.device.isConnected && !props.device.isConnecting
}));

const statusIndicatorClass = computed(() => ({
    'status-connected': props.device.isConnected,
    'status-connecting': props.device.isConnecting,
    'status-disconnected': !props.device.isConnected && !props.device.isConnecting
}));

const statusEmoji = computed(() => {
    if (props.device.isConnecting) return '⏳';
    if (props.device.isConnected) return '🟢';
    return '🔴';
});

const connectButtonClass = computed(() => ({
    'btn-success': props.device.isConnected,
    'btn-primary': !props.device.isConnected && !props.device.isConnecting,
    'btn-loading': props.device.isConnecting
}));

const customColorStyle = computed(() => {
    if (!props.device.customColor) return {};
    const { r, g, b } = props.device.customColor;
    return {
        backgroundColor: `rgb(${Math.round(r * 255)}, ${Math.round(g * 255)}, ${Math.round(b * 255)})`
    };
});

// Event handlers
const handleConnect = () => {
    emit('connect', props.device.id);
};

const handleDisconnect = () => {
    emit('disconnect', props.device.id);
};

const handlePing = () => {
    emit('ping', props.device.id);
};

const handleEffectChange = (effectId: number) => {
    emit('setEffect', props.device.id, effectId);
};

const handleColorModeChange = (mode: string) => {
    emit('setColorMode', props.device.id, mode);
};

const handleRemove = () => {
    if (confirm(`Êtes-vous sûr de vouloir supprimer "${props.device.name}" ?`)) {
        emit('remove', props.device.id);
    }
};

// Utility functions
const formatLastSeen = (date: Date): string => {
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const minutes = Math.floor(diff / 60000);
    
    if (minutes < 1) return 'À l\'instant';
    if (minutes < 60) return `${minutes}min`;
    
    const hours = Math.floor(minutes / 60);
    if (hours < 24) return `${hours}h`;
    
    const days = Math.floor(hours / 24);
    return `${days}j`;
};
</script>

<style scoped>
.device-panel {
    background: #161b22;
    border: 1px solid #30363d;
    border-radius: 12px;
    padding: 1rem;
    transition: all 0.3s ease;
    position: relative;
    overflow: hidden;
}

.device-panel:hover {
    border-color: #58a6ff;
    box-shadow: 0 0 0 1px #58a6ff;
}

.device-panel.device-connected {
    border-color: #238636;
    background: linear-gradient(135deg, #161b22 0%, #0d1117 100%);
}

.device-panel.device-connecting {
    border-color: #f85149;
    animation: pulse 2s infinite;
}

.device-panel.device-disconnected {
    opacity: 0.8;
}

/* Header */
.device-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 1rem;
}

.device-info {
    flex: 1;
}

.device-name {
    font-size: 1.1rem;
    font-weight: 600;
    color: #f0f6fc;
    margin: 0 0 0.25rem 0;
}

.device-address {
    font-size: 0.85rem;
    color: #8b949e;
    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
}

.device-status {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.25rem;
}

.status-indicator {
    font-size: 1.2rem;
    animation: none;
}

.status-indicator.status-connecting {
    animation: pulse 1s infinite;
}

.ping-info {
    font-size: 0.75rem;
    color: #7c3aed;
    font-weight: 500;
}

/* Connection Controls */
.connection-controls {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1rem;
    flex-wrap: wrap;
}

/* Quick Controls */
.quick-controls {
    border-top: 1px solid #30363d;
    padding-top: 1rem;
    margin-bottom: 1rem;
}

.control-section {
    margin-bottom: 1rem;
}

.control-section:last-child {
    margin-bottom: 0;
}

.control-section h4 {
    font-size: 0.85rem;
    color: #f0f6fc;
    margin: 0 0 0.5rem 0;
    font-weight: 500;
}

.effect-buttons,
.color-buttons {
    display: flex;
    gap: 0.25rem;
    flex-wrap: wrap;
}

.effect-btn,
.color-btn {
    min-width: 2.5rem;
    height: 2.5rem;
    padding: 0;
    font-size: 1rem;
}

.color-preview {
    width: 100%;
    height: 2rem;
    border-radius: 4px;
    border: 1px solid #30363d;
}

/* Device Settings */
.device-settings {
    display: flex;
    gap: 0.5rem;
    border-top: 1px solid #30363d;
    padding-top: 1rem;
    margin-bottom: 1rem;
}

/* Footer */
.device-footer {
    text-align: center;
    border-top: 1px solid #30363d;
    padding-top: 0.5rem;
}

.last-seen {
    color: #8b949e;
    font-size: 0.75rem;
}

/* Button Styles */
.btn {
    padding: 0.5rem 1rem;
    border: 1px solid #30363d;
    background: #21262d;
    color: #f0f6fc;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.875rem;
    font-weight: 500;
    transition: all 0.2s ease;
    text-align: center;
    text-decoration: none;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 0.25rem;
    flex: 1;
    min-width: 0;
}

.btn:hover:not(:disabled) {
    background: #30363d;
    border-color: #58a6ff;
}

.btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

.btn-small {
    padding: 0.375rem 0.75rem;
    font-size: 0.75rem;
    flex: none;
}

.btn-primary {
    background: #238636;
    border-color: #238636;
}

.btn-primary:hover:not(:disabled) {
    background: #2ea043;
}

.btn-success {
    background: #1f6feb;
    border-color: #1f6feb;
}

.btn-success:hover:not(:disabled) {
    background: #388bfd;
}

.btn-secondary {
    background: #6e7681;
    border-color: #6e7681;
}

.btn-secondary:hover:not(:disabled) {
    background: #8b949e;
}

.btn-info {
    background: #7c3aed;
    border-color: #7c3aed;
}

.btn-info:hover:not(:disabled) {
    background: #8b5cf6;
}

.btn-danger {
    background: #da3633;
    border-color: #da3633;
}

.btn-danger:hover:not(:disabled) {
    background: #f85149;
}

.btn.active {
    background: #58a6ff;
    border-color: #58a6ff;
    color: #0d1117;
}

/* Animations */
@keyframes pulse {
    0%, 100% {
        opacity: 1;
    }
    50% {
        opacity: 0.5;
    }
}

/* Responsive */
@media (max-width: 480px) {
    .device-panel {
        padding: 0.75rem;
    }
    
    .connection-controls {
        flex-direction: column;
    }
    
    .device-settings {
        flex-direction: column;
    }
    
    .effect-buttons,
    .color-buttons {
        justify-content: center;
    }
}
</style> 