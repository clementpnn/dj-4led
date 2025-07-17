<!-- src/components/DeviceGrid.vue -->
<template>
    <div class="device-grid-container" ref="containerRef">
        <!-- Grid Header -->
        <div class="grid-header">
            <div class="grid-info">
                <h2 class="grid-title">🎛️ Valises DJ-4LED</h2>
                <span class="device-count">{{ devices.length }} {{ devices.length === 1 ? 'valise' : 'valises' }}</span>
                <span v-if="connectedCount > 0" class="connected-count">
                    ({{ connectedCount }} {{ connectedCount === 1 ? 'connectée' : 'connectées' }})
                </span>
            </div>

            <!-- Grid Controls -->
            <div class="grid-controls">
                <!-- Layout Controls -->
                <div class="layout-controls">
                    <button 
                        class="btn btn-small" 
                        @click="changeColumns(-1)"
                        :disabled="gridSettings.columns <= 1"
                        title="Moins de colonnes"
                    >
                        ⬅️
                    </button>
                    <span class="column-count">{{ gridSettings.columns }}</span>
                    <button 
                        class="btn btn-small" 
                        @click="changeColumns(1)"
                        :disabled="gridSettings.columns >= maxColumns"
                        title="Plus de colonnes"
                    >
                        ➡️
                    </button>
                </div>

                <!-- Auto-fit toggle -->
                <button 
                    class="btn btn-small"
                    :class="{ active: autoFit }"
                    @click="toggleAutoFit"
                    title="Ajustement automatique"
                >
                    🔄 Auto
                </button>

                <!-- Bulk Actions -->
                <div class="bulk-actions">
                    <button 
                        class="btn btn-small btn-success"
                        @click="$emit('connectAll')"
                        :disabled="allConnected"
                        title="Connecter toutes"
                    >
                        🔌 Toutes
                    </button>
                    <button 
                        class="btn btn-small btn-secondary"
                        @click="$emit('disconnectAll')"
                        :disabled="!hasConnected"
                        title="Déconnecter toutes"
                    >
                        🔌❌ Aucune
                    </button>
                    <button 
                        class="btn btn-small btn-info"
                        @click="$emit('pingAll')"
                        title="Ping toutes"
                    >
                        🏓 Ping
                    </button>
                </div>

                <!-- Add Device Button -->
                <button 
                    class="btn btn-small btn-primary"
                    @click="$emit('addDevice')"
                    title="Ajouter une valise"
                >
                    ➕ Ajouter
                </button>
            </div>
        </div>

        <!-- Grid Content -->
        <div 
            class="device-grid"
            :style="gridStyle"
            :class="{ 'is-resizable': gridSettings.isResizable }"
        >
            <DevicePanel
                v-for="device in devices"
                :key="device.id"
                :device="device"
                :effects="effects"
                :color-modes="colorModes"
                @connect="(id) => $emit('connectDevice', id)"
                @disconnect="(id) => $emit('disconnectDevice', id)"
                @ping="(id) => $emit('pingDevice', id)"
                @set-effect="(id, effectId) => $emit('setDeviceEffect', id, effectId)"
                @set-color-mode="(id, mode) => $emit('setDeviceColorMode', id, mode)"
                @remove="(id) => $emit('removeDevice', id)"
                @edit="(id) => $emit('editDevice', id)"
            />

            <!-- Empty State -->
            <div v-if="devices.length === 0" class="empty-state">
                <div class="empty-content">
                    <div class="empty-icon">🎛️</div>
                    <h3>Aucune valise configurée</h3>
                    <p>Ajoutez votre première valise DJ-4LED pour commencer</p>
                    <button 
                        class="btn btn-primary"
                        @click="$emit('addDevice')"
                    >
                        ➕ Ajouter une valise
                    </button>
                </div>
            </div>
        </div>

        <!-- Grid Footer -->
        <div v-if="devices.length > 0" class="grid-footer">
            <div class="grid-stats">
                <span class="stat">
                    📊 {{ devices.length }} valise{{ devices.length > 1 ? 's' : '' }}
                </span>
                <span class="stat">
                    🟢 {{ connectedCount }} connectée{{ connectedCount > 1 ? 's' : '' }}
                </span>
                <span class="stat" v-if="avgPing > 0">
                    🏓 {{ avgPing }}ms moyen
                </span>
            </div>

            <!-- Resize Handle (if resizable) -->
            <div v-if="gridSettings.isResizable" class="resize-controls">
                <button 
                    class="btn btn-small"
                    @click="resetGrid"
                    title="Réinitialiser la grille"
                >
                    🔄 Reset
                </button>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted, watch } from 'vue';
import DevicePanel from './DevicePanel.vue';

// Types
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

interface GridSettings {
    columns: number;
    itemMinWidth: number;
    itemMaxWidth: number;
    gap: number;
    isResizable: boolean;
}

// Props
interface Props {
    devices: Device[];
    gridSettings: GridSettings;
    effects?: Effect[];
    colorModes?: ColorMode[];
}

const props = withDefaults(defineProps<Props>(), {
    effects: () => [],
    colorModes: () => []
});

// Emits
interface Emits {
    updateGridSettings: [settings: Partial<GridSettings>];
    connectDevice: [deviceId: string];
    disconnectDevice: [deviceId: string];
    pingDevice: [deviceId: string];
    setDeviceEffect: [deviceId: string, effectId: number];
    setDeviceColorMode: [deviceId: string, mode: string];
    removeDevice: [deviceId: string];
    editDevice: [deviceId: string];
    addDevice: [];
    connectAll: [];
    disconnectAll: [];
    pingAll: [];
}

const emit = defineEmits<Emits>();

// Refs
const containerRef = ref<HTMLElement>();
const autoFit = ref(true);

// Computed properties
const connectedCount = computed(() => 
    props.devices.filter(device => device.isConnected).length
);

const allConnected = computed(() => 
    props.devices.length > 0 && connectedCount.value === props.devices.length
);

const hasConnected = computed(() => connectedCount.value > 0);

const avgPing = computed(() => {
    const pings = props.devices
        .filter(device => device.lastPing !== undefined)
        .map(device => device.lastPing!);
    
    if (pings.length === 0) return 0;
    return Math.round(pings.reduce((sum, ping) => sum + ping, 0) / pings.length);
});

const maxColumns = computed(() => Math.min(6, props.devices.length || 1));

const gridStyle = computed(() => {
    const { columns, itemMinWidth, itemMaxWidth, gap } = props.gridSettings;
    
    return {
        display: 'grid',
        gridTemplateColumns: autoFit.value 
            ? `repeat(auto-fit, minmax(${itemMinWidth}px, ${itemMaxWidth}px))`
            : `repeat(${columns}, 1fr)`,
        gap: `${gap}px`,
        justifyContent: autoFit.value ? 'center' : 'stretch'
    };
});

// Methods
const changeColumns = (delta: number) => {
    const newColumns = Math.max(1, Math.min(maxColumns.value, props.gridSettings.columns + delta));
    emit('updateGridSettings', { columns: newColumns });
};

const toggleAutoFit = () => {
    autoFit.value = !autoFit.value;
    if (autoFit.value) {
        updateColumnsFromContainer();
    }
};

const updateColumnsFromContainer = () => {
    if (!containerRef.value || !autoFit.value) return;
    
    const containerWidth = containerRef.value.clientWidth;
    const { itemMinWidth, gap } = props.gridSettings;
    const availableWidth = containerWidth - gap;
    const itemWidthWithGap = itemMinWidth + gap;
    const optimalColumns = Math.max(1, Math.floor(availableWidth / itemWidthWithGap));
    
    if (optimalColumns !== props.gridSettings.columns) {
        emit('updateGridSettings', { columns: optimalColumns });
    }
};

const resetGrid = () => {
    emit('updateGridSettings', {
        columns: 3,
        itemMinWidth: 300,
        itemMaxWidth: 500,
        gap: 16
    });
    autoFit.value = true;
};

// Resize observer
let resizeObserver: ResizeObserver | null = null;

onMounted(() => {
    if (containerRef.value) {
        resizeObserver = new ResizeObserver(() => {
            updateColumnsFromContainer();
        });
        resizeObserver.observe(containerRef.value);
        updateColumnsFromContainer();
    }
});

onUnmounted(() => {
    if (resizeObserver) {
        resizeObserver.disconnect();
    }
});

// Watch for device changes
watch(() => props.devices.length, () => {
    if (autoFit.value) {
        updateColumnsFromContainer();
    }
});
</script>

<style scoped>
.device-grid-container {
    width: 100%;
    max-width: 1400px;
    margin: 0 auto;
    padding: 1rem;
}

/* Grid Header */
.grid-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
    padding: 1rem;
    background: #161b22;
    border: 1px solid #30363d;
    border-radius: 12px;
    flex-wrap: wrap;
    gap: 1rem;
}

.grid-info {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex-wrap: wrap;
}

.grid-title {
    font-size: 1.25rem;
    font-weight: 600;
    color: #f0f6fc;
    margin: 0;
}

.device-count {
    color: #58a6ff;
    font-weight: 500;
}

.connected-count {
    color: #238636;
    font-weight: 500;
}

.grid-controls {
    display: flex;
    align-items: center;
    gap: 1rem;
    flex-wrap: wrap;
}

.layout-controls {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.25rem 0.5rem;
    background: #21262d;
    border-radius: 6px;
    border: 1px solid #30363d;
}

.column-count {
    color: #f0f6fc;
    font-weight: 500;
    min-width: 1.5rem;
    text-align: center;
}

.bulk-actions {
    display: flex;
    gap: 0.25rem;
}

/* Grid Content */
.device-grid {
    width: 100%;
    margin-bottom: 1.5rem;
    min-height: 200px;
}

.device-grid.is-resizable {
    resize: both;
    overflow: auto;
    border: 2px dashed #30363d;
    border-radius: 8px;
    padding: 1rem;
}

/* Empty State */
.empty-state {
    grid-column: 1 / -1;
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 300px;
    padding: 2rem;
}

.empty-content {
    text-align: center;
    max-width: 400px;
}

.empty-icon {
    font-size: 4rem;
    margin-bottom: 1rem;
    opacity: 0.5;
}

.empty-content h3 {
    color: #f0f6fc;
    margin: 0 0 0.5rem 0;
    font-size: 1.25rem;
}

.empty-content p {
    color: #8b949e;
    margin: 0 0 1.5rem 0;
    line-height: 1.5;
}

/* Grid Footer */
.grid-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    background: #161b22;
    border: 1px solid #30363d;
    border-radius: 12px;
    flex-wrap: wrap;
    gap: 1rem;
}

.grid-stats {
    display: flex;
    gap: 1rem;
    flex-wrap: wrap;
}

.stat {
    color: #8b949e;
    font-size: 0.875rem;
}

.resize-controls {
    display: flex;
    gap: 0.5rem;
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

.btn.active {
    background: #58a6ff;
    border-color: #58a6ff;
    color: #0d1117;
}

/* Responsive Design */
@media (max-width: 1200px) {
    .device-grid-container {
        padding: 0.75rem;
    }
    
    .grid-header {
        flex-direction: column;
        align-items: stretch;
    }
    
    .grid-controls {
        justify-content: center;
    }
}

@media (max-width: 768px) {
    .device-grid-container {
        padding: 0.5rem;
    }
    
    .grid-info {
        justify-content: center;
        text-align: center;
    }
    
    .grid-controls {
        flex-direction: column;
        gap: 0.5rem;
    }
    
    .bulk-actions {
        flex-wrap: wrap;
        justify-content: center;
    }
    
    .device-grid {
        grid-template-columns: 1fr !important;
    }
}

@media (max-width: 480px) {
    .grid-header,
    .grid-footer {
        padding: 0.75rem;
    }
    
    .grid-title {
        font-size: 1.1rem;
    }
    
    .grid-stats {
        flex-direction: column;
        gap: 0.25rem;
        text-align: center;
    }
}
</style> 