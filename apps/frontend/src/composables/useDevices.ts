// src/composables/useDevices.ts
import { invoke } from '@tauri-apps/api/core';
import { ref, computed } from 'vue';
import type { Device, DeviceResult, DeviceGridSettings } from '../types';

export function useDevices() {
    const devices = ref<Device[]>([]);
    const selectedDeviceId = ref<string | null>(null);
    const gridSettings = ref<DeviceGridSettings>({
        columns: 3,
        itemMinWidth: 300,
        itemMaxWidth: 500,
        gap: 16,
        isResizable: true
    });

    // Computed properties
    const connectedDevices = computed(() => 
        devices.value.filter(device => device.isConnected)
    );

    const selectedDevice = computed(() => 
        devices.value.find(device => device.id === selectedDeviceId.value) || null
    );

    const deviceCount = computed(() => devices.value.length);

    // Device management functions
    const generateDeviceId = (): string => {
        return `device_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    };

    const addDevice = (name: string, ipAddress: string, port: number = 8081): Device => {
        const device: Device = {
            id: generateDeviceId(),
            name,
            ipAddress,
            port,
            isConnected: false,
            isConnecting: false
        };

        devices.value.push(device);
        return device;
    };

    const removeDevice = (deviceId: string): boolean => {
        const index = devices.value.findIndex(device => device.id === deviceId);
        if (index !== -1) {
            // Disconnect if connected
            const device = devices.value[index];
            if (device.isConnected) {
                disconnectDevice(device.id);
            }
            devices.value.splice(index, 1);
            
            // Update selected device if it was removed
            if (selectedDeviceId.value === deviceId) {
                selectedDeviceId.value = devices.value.length > 0 ? devices.value[0].id : null;
            }
            return true;
        }
        return false;
    };

    const updateDevice = (deviceId: string, updates: Partial<Device>): boolean => {
        const device = devices.value.find(d => d.id === deviceId);
        if (device) {
            Object.assign(device, updates);
            return true;
        }
        return false;
    };

    const selectDevice = (deviceId: string): void => {
        if (devices.value.find(device => device.id === deviceId)) {
            selectedDeviceId.value = deviceId;
        }
    };

    // Connection functions
    const connectDevice = async (deviceId: string): Promise<DeviceResult> => {
        const device = devices.value.find(d => d.id === deviceId);
        if (!device) {
            return { success: false, message: "Device not found" };
        }

        device.isConnecting = true;
        
        try {
            const serverAddress = `${device.ipAddress}:${device.port}`;
            const result = await invoke<string>('dj_connect', { serverAddress });
            
            const success = result.includes('✅');
            device.isConnected = success;
            device.lastSeen = success ? new Date() : device.lastSeen;
            
            return {
                success,
                message: result,
                device
            };
        } catch (error) {
            device.isConnected = false;
            return {
                success: false,
                message: `❌ Connection error: ${error}`,
                device
            };
        } finally {
            device.isConnecting = false;
        }
    };

    const disconnectDevice = async (deviceId: string): Promise<DeviceResult> => {
        const device = devices.value.find(d => d.id === deviceId);
        if (!device) {
            return { success: false, message: "Device not found" };
        }

        try {
            const serverAddress = `${device.ipAddress}:${device.port}`;
            const result = await invoke<string>('dj_disconnect', { serverAddress });
            
            device.isConnected = false;
            device.isConnecting = false;
            device.currentEffect = undefined;
            device.currentColorMode = undefined;
            device.customColor = undefined;
            
            return {
                success: true,
                message: result,
                device
            };
        } catch (error) {
            return {
                success: false,
                message: `❌ Disconnection error: ${error}`,
                device
            };
        }
    };

    const pingDevice = async (deviceId: string): Promise<DeviceResult> => {
        const device = devices.value.find(d => d.id === deviceId);
        if (!device) {
            return { success: false, message: "Device not found" };
        }

        const startTime = performance.now();
        
        try {
            const serverAddress = `${device.ipAddress}:${device.port}`;
            const result = await invoke<string>('dj_ping', { serverAddress });
            
            const endTime = performance.now();
            const pingMs = Math.round(endTime - startTime);
            
            const success = result.includes('🏓');
            device.lastPing = success ? pingMs : undefined;
            device.lastSeen = success ? new Date() : device.lastSeen;
            
            return {
                success,
                message: result,
                device
            };
        } catch (error) {
            device.lastPing = undefined;
            return {
                success: false,
                message: `❌ Ping failed: ${error}`,
                device
            };
        }
    };

    // Control functions for specific device
    const setDeviceEffect = async (deviceId: string, effectId: number): Promise<DeviceResult> => {
        const device = devices.value.find(d => d.id === deviceId);
        if (!device || !device.isConnected) {
            return { success: false, message: "Device not connected" };
        }

        try {
            const serverAddress = `${device.ipAddress}:${device.port}`;
            const result = await invoke<string>('dj_set_effect', { serverAddress, effectId });
            
            device.currentEffect = effectId;
            return {
                success: true,
                message: result,
                device
            };
        } catch (error) {
            return {
                success: false,
                message: `❌ Effect error: ${error}`,
                device
            };
        }
    };

    const setDeviceColorMode = async (deviceId: string, mode: string): Promise<DeviceResult> => {
        const device = devices.value.find(d => d.id === deviceId);
        if (!device || !device.isConnected) {
            return { success: false, message: "Device not connected" };
        }

        try {
            const serverAddress = `${device.ipAddress}:${device.port}`;
            const result = await invoke<string>('dj_set_color_mode', { serverAddress, mode });
            
            device.currentColorMode = mode;
            return {
                success: true,
                message: result,
                device
            };
        } catch (error) {
            return {
                success: false,
                message: `❌ Color mode error: ${error}`,
                device
            };
        }
    };

    const setDeviceCustomColor = async (deviceId: string, r: number, g: number, b: number): Promise<DeviceResult> => {
        const device = devices.value.find(d => d.id === deviceId);
        if (!device || !device.isConnected) {
            return { success: false, message: "Device not connected" };
        }

        try {
            const serverAddress = `${device.ipAddress}:${device.port}`;
            const result = await invoke<string>('dj_set_custom_color', { serverAddress, r, g, b });
            
            device.customColor = { r, g, b };
            return {
                success: true,
                message: result,
                device
            };
        } catch (error) {
            return {
                success: false,
                message: `❌ Custom color error: ${error}`,
                device
            };
        }
    };

    const listenDeviceData = async (deviceId: string): Promise<DeviceResult> => {
        const device = devices.value.find(d => d.id === deviceId);
        if (!device || !device.isConnected) {
            return { success: false, message: "Device not connected" };
        }

        try {
            const serverAddress = `${device.ipAddress}:${device.port}`;
            const result = await invoke<string>('dj_listen_data', { serverAddress });
            
            return {
                success: true,
                message: result,
                device
            };
        } catch (error) {
            return {
                success: false,
                message: `❌ Listen error: ${error}`,
                device
            };
        }
    };

    // Grid management
    const updateGridSettings = (newSettings: Partial<DeviceGridSettings>): void => {
        gridSettings.value = { ...gridSettings.value, ...newSettings };
    };

    const calculateOptimalColumns = (containerWidth: number): number => {
        const { itemMinWidth, gap } = gridSettings.value;
        const availableWidth = containerWidth - gap;
        const itemWidthWithGap = itemMinWidth + gap;
        return Math.max(1, Math.floor(availableWidth / itemWidthWithGap));
    };

    // Bulk operations
    const connectAllDevices = async (): Promise<DeviceResult[]> => {
        const results: DeviceResult[] = [];
        for (const device of devices.value) {
            if (!device.isConnected) {
                const result = await connectDevice(device.id);
                results.push(result);
            }
        }
        return results;
    };

    const disconnectAllDevices = async (): Promise<DeviceResult[]> => {
        const results: DeviceResult[] = [];
        for (const device of devices.value) {
            if (device.isConnected) {
                const result = await disconnectDevice(device.id);
                results.push(result);
            }
        }
        return results;
    };

    const pingAllDevices = async (): Promise<DeviceResult[]> => {
        const results: DeviceResult[] = [];
        for (const device of devices.value) {
            const result = await pingDevice(device.id);
            results.push(result);
        }
        return results;
    };

    // Initialize with some default devices (can be removed or moved to config)
    const initializeDefaultDevices = (): void => {
        addDevice("Valise 1", "192.168.1.45");
        addDevice("Valise 2", "192.168.1.46");
        addDevice("Valise 3", "192.168.1.47");
        addDevice("Valise 4", "192.168.1.48");
        
        if (devices.value.length > 0) {
            selectedDeviceId.value = devices.value[0].id;
        }
    };

    return {
        // State
        devices,
        selectedDeviceId,
        gridSettings,
        
        // Computed
        connectedDevices,
        selectedDevice,
        deviceCount,
        
        // Device management
        addDevice,
        removeDevice,
        updateDevice,
        selectDevice,
        
        // Connection operations
        connectDevice,
        disconnectDevice,
        pingDevice,
        
        // Control operations
        setDeviceEffect,
        setDeviceColorMode,
        setDeviceCustomColor,
        listenDeviceData,
        
        // Grid management
        updateGridSettings,
        calculateOptimalColumns,
        
        // Bulk operations
        connectAllDevices,
        disconnectAllDevices,
        pingAllDevices,
        
        // Initialization
        initializeDefaultDevices
    };
} 