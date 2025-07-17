<!-- src/App.vue -->
<template>
    <div class="app">
        <!-- Header -->
        <Header :is-connected="hasConnectedDevices" />

        <!-- Main content -->
        <div class="main-content">
            <!-- Device Grid - New multi-device system -->
            <DeviceGrid
                :devices="devices.devices.value"
                :grid-settings="devices.gridSettings.value"
                :effects="EFFECTS"
                :color-modes="COLOR_MODES"
                @update-grid-settings="devices.updateGridSettings"
                @connect-device="handleDeviceConnect"
                @disconnect-device="handleDeviceDisconnect"
                @ping-device="handleDevicePing"
                @set-device-effect="handleDeviceEffect"
                @set-device-color-mode="handleDeviceColorMode"
                @remove-device="handleDeviceRemove"
                @edit-device="handleDeviceEdit"
                @add-device="handleDeviceAdd"
                @connect-all="handleConnectAll"
                @disconnect-all="handleDisconnectAll"
                @ping-all="handlePingAll"
            />

            <!-- Device Modal -->
            <DeviceModal
                :visible="showDeviceModal"
                :device="editingDevice"
                @close="closeDeviceModal"
                @save="handleDeviceSave"
                @test="handleDeviceTest"
            />

            <!-- Real-time data display (only for selected/first connected device) -->
            <DataPanel
                v-if="streaming.streamData.value.frames.length > 0 || streaming.streamData.value.spectrum.length > 0"
                :stream-data="streaming.streamData.value"
            />

            <!-- Global Control panels (affect selected device) -->
            <div v-if="devices.selectedDevice.value" class="global-controls">
                <div class="global-controls-header">
                    <h3>🎮 Contrôles pour: {{ devices.selectedDevice.value.name }}</h3>
                    <div class="device-selector">
                        <select 
                            v-model="devices.selectedDeviceId.value"
                            class="device-select"
                        >
                            <option 
                                v-for="device in devices.connectedDevices.value" 
                                :key="device.id"
                                :value="device.id"
                            >
                                {{ device.name }} ({{ device.ipAddress }})
                            </option>
                        </select>
                    </div>
                </div>

                <div class="control-grid">
                    <!-- Effects panel -->
                    <EffectsPanel
                        :effects="EFFECTS"
                        :current-effect="devices.selectedDevice.value?.currentEffect ?? null"
                        :is-connected="devices.selectedDevice.value?.isConnected ?? false"
                        :loading="false"
                        @effect-change="(effectId) => devices.selectedDevice.value && handleDeviceEffect(devices.selectedDevice.value.id, effectId)"
                    />

                    <!-- Color modes panel -->
                    <ColorModesPanel
                        :color-modes="COLOR_MODES"
                        :current-mode="devices.selectedDevice.value?.currentColorMode ?? null"
                        :is-connected="devices.selectedDevice.value?.isConnected ?? false"
                        :loading="false"
                        @mode-change="(mode) => devices.selectedDevice.value && handleDeviceColorMode(devices.selectedDevice.value.id, mode)"
                    />

                    <!-- Custom color panel -->
                    <CustomColorPanel
                        :custom-color="devices.selectedDevice.value?.customColor || { r: 1, g: 1, b: 1 }"
                        :color-channels="COLOR_CHANNELS"
                        :color-preview-style="selectedDeviceColorStyle"
                        :is-connected="devices.selectedDevice.value?.isConnected ?? false"
                        :loading="false"
                        @color-apply="handleSelectedDeviceColorApply"
                        @color-update="handleSelectedDeviceColorUpdate"
                    />
                </div>
            </div>

            <!-- Console terminal -->
            <Terminal :logs="logs.logs.value" @clear-logs="logs.clearLogs" ref="terminalRef" />
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';

// Components
import {
    ColorModesPanel,
    CustomColorPanel,
    DataPanel,
    DeviceGrid,
    DeviceModal,
    EffectsPanel,
    Header,
    Terminal
} from "@monorepo/ui";

// Composables
import { useDevices } from './composables/useDevices';
import { useLogs } from './composables/useLogs';
import { useStreaming } from './composables/useStreaming';

// Constants
import { COLOR_CHANNELS, COLOR_MODES, EFFECTS } from './utils/constants';

// Types
import type { Device } from './types';

// Composables initialization
const devices = useDevices();
const logs = useLogs();
const streaming = useStreaming();

// Refs
const terminalRef = ref<InstanceType<typeof Terminal> | null>(null);
const showDeviceModal = ref(false);
const editingDevice = ref<Device | undefined>(undefined);

// Computed properties
const hasConnectedDevices = computed(() => devices.connectedDevices.value.length > 0);

const selectedDeviceColorStyle = computed(() => {
    const device = devices.selectedDevice.value;
    if (!device?.customColor) {
        return { backgroundColor: 'rgb(255, 255, 255)' };
    }
    const { r, g, b } = device.customColor;
    return {
        backgroundColor: `rgb(${Math.round(r * 255)}, ${Math.round(g * 255)}, ${Math.round(b * 255)})`
    };
});

// Device Management Handlers
const handleDeviceAdd = () => {
    editingDevice.value = undefined;
    showDeviceModal.value = true;
};

const handleDeviceEdit = (deviceId: string) => {
    const device = devices.devices.value.find(d => d.id === deviceId);
    if (device) {
        editingDevice.value = device;
        showDeviceModal.value = true;
    }
};

const closeDeviceModal = () => {
    showDeviceModal.value = false;
    editingDevice.value = undefined;
};

const handleDeviceSave = (deviceData: { name: string; ipAddress: string; port: number }) => {
    if (editingDevice.value?.id) {
        // Update existing device
        devices.updateDevice(editingDevice.value.id, deviceData);
        logs.log(`✏️ Valise "${deviceData.name}" modifiée`, 'info');
    } else {
        // Add new device
        const newDevice = devices.addDevice(deviceData.name, deviceData.ipAddress, deviceData.port);
        logs.log(`➕ Nouvelle valise "${newDevice.name}" ajoutée (${newDevice.ipAddress}:${newDevice.port})`, 'success');
        
        // Auto-select first device if none selected
        if (!devices.selectedDeviceId.value) {
            devices.selectDevice(newDevice.id);
        }
    }
    closeDeviceModal();
};

const handleDeviceRemove = (deviceId: string) => {
    const device = devices.devices.value.find(d => d.id === deviceId);
    if (device) {
        devices.removeDevice(deviceId);
        logs.log(`🗑️ Valise "${device.name}" supprimée`, 'warning');
    }
};

const handleDeviceTest = async (deviceData: { name: string; ipAddress: string; port: number }) => {
    logs.log(`🧪 Test de connexion: ${deviceData.ipAddress}:${deviceData.port}...`, 'info');
    // Note: The actual test will be handled by the modal's internal logic
    // This is just for logging purposes
};

// Connection Handlers
const handleDeviceConnect = async (deviceId: string) => {
    const result = await devices.connectDevice(deviceId);
    logs.log(result.message, result.success ? 'success' : 'error');
    
    if (result.success && result.device) {
        // Auto-select connected device if none selected
        if (!devices.selectedDeviceId.value) {
            devices.selectDevice(deviceId);
        }
    }
};

const handleDeviceDisconnect = async (deviceId: string) => {
    const result = await devices.disconnectDevice(deviceId);
    logs.log(result.message, result.success ? 'success' : 'warning');
};

const handleDevicePing = async (deviceId: string) => {
    const result = await devices.pingDevice(deviceId);
    logs.log(result.message, result.success ? 'success' : 'warning');
};

// Bulk Connection Handlers
const handleConnectAll = async () => {
    logs.log('🔌 Connexion de toutes les valises...', 'info');
    const results = await devices.connectAllDevices();
    const successCount = results.filter(r => r.success).length;
    logs.log(`✅ ${successCount}/${results.length} valises connectées`, 'success');
};

const handleDisconnectAll = async () => {
    logs.log('🔌❌ Déconnexion de toutes les valises...', 'info');
    const results = await devices.disconnectAllDevices();
    logs.log(`✅ Toutes les valises déconnectées`, 'success');
};

const handlePingAll = async () => {
    logs.log('🏓 Ping de toutes les valises...', 'info');
    const results = await devices.pingAllDevices();
    const successCount = results.filter(r => r.success).length;
    logs.log(`🏓 ${successCount}/${results.length} valises répondent`, 'info');
};

// Control Handlers
const handleDeviceEffect = async (deviceId: string, effectId: number) => {
    const result = await devices.setDeviceEffect(deviceId, effectId);
    logs.log(result.message, result.success ? 'success' : 'error');
};

const handleDeviceColorMode = async (deviceId: string, mode: string) => {
    const result = await devices.setDeviceColorMode(deviceId, mode);
    logs.log(result.message, result.success ? 'success' : 'error');
};

// Selected Device Color Handlers
const handleSelectedDeviceColorApply = async () => {
    const device = devices.selectedDevice.value;
    if (!device?.customColor) return;
    
    const { r, g, b } = device.customColor;
    logs.log(`🎨 Application RGB(${r.toFixed(2)}, ${g.toFixed(2)}, ${b.toFixed(2)}) sur "${device.name}"...`, 'info');
    const result = await devices.setDeviceCustomColor(device.id, r, g, b);
    logs.log(result.message, result.success ? 'success' : 'error');
};

const handleSelectedDeviceColorUpdate = (newColor: { r: number; g: number; b: number }) => {
    const device = devices.selectedDevice.value;
    if (device) {
        devices.updateDevice(device.id, { customColor: newColor });
    }
};

// Streaming Handler
const handleStream = async () => {
    const device = devices.selectedDevice.value;
    if (!device) {
        logs.log('❌ Aucune valise sélectionnée pour le streaming', 'error');
        return;
    }
    
    logs.log(`📡 Écoute du stream depuis "${device.name}"...`, 'info');
    const result = await devices.listenDeviceData(device.id);
    logs.log(result.message, result.success ? 'success' : 'error');
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

// Initialize
onMounted(() => {
    logs.initLogs();
    
    // Set the log container reference
    if (terminalRef.value?.logContainer) {
        logs.logContainer.value = terminalRef.value.logContainer;
    }
    
    // Initialize with default devices
    devices.initializeDefaultDevices();
    
    logs.log('🎛️ Application DJ-4LED démarrée', 'info');
    logs.log(`📱 ${devices.devices.value.length} valises configurées`, 'info');
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
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
    background: #0d1117;
    color: #f0f6fc;
    overflow-x: hidden;
}

/* Main content */
.main-content {
    max-width: 1600px;
    margin: 0 auto;
    padding: 2rem;
}

/* Global Controls */
.global-controls {
    margin: 2rem 0;
    padding: 1.5rem;
    background: #161b22;
    border: 1px solid #30363d;
    border-radius: 12px;
}

.global-controls-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
    flex-wrap: wrap;
    gap: 1rem;
}

.global-controls-header h3 {
    color: #f0f6fc;
    margin: 0;
    font-size: 1.1rem;
}

.device-selector {
    display: flex;
    align-items: center;
    gap: 0.5rem;
}

.device-select {
    padding: 0.5rem 1rem;
    background: #21262d;
    border: 1px solid #30363d;
    border-radius: 6px;
    color: #f0f6fc;
    font-size: 0.875rem;
    min-width: 200px;
    cursor: pointer;
}

.device-select:focus {
    outline: none;
    border-color: #58a6ff;
}

/* Control grid */
.control-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(350px, 1fr));
    gap: 1.5rem;
}

/* Responsive */
@media (max-width: 1200px) {
    .main-content {
        padding: 1rem;
    }
    
    .global-controls-header {
        flex-direction: column;
        align-items: stretch;
    }
    
    .device-selector {
        justify-content: center;
    }
}

@media (max-width: 768px) {
    .main-content {
        padding: 0.75rem;
    }
    
    .control-grid {
        grid-template-columns: 1fr;
    }
    
    .device-select {
        min-width: 100%;
    }
}
</style>
