// src/types/index.ts

export interface CustomColor {
    r: number;
    g: number;
    b: number;
}

export interface FrameData {
    width: number;
    height: number;
    format: number;
    size: number;
    timestamp: number;
}

export interface StreamData {
    frames: FrameData[];
    spectrum: number[];
    lastFrame: FrameData | null;
}

export interface LogEntry {
    time: string;
    message: string;
    type: 'info' | 'success' | 'error' | 'warning';
}

export interface Effect {
    id: number;
    name: string;
    emoji: string;
}

export interface ColorMode {
    value: string;
    label: string;
    emoji: string;
}

export interface ColorChannel {
    key: 'r' | 'g' | 'b';
    name: string;
    emoji: string;
}

// Result interfaces for async operations
export interface ConnectionResult {
    success: boolean;
    message: string;
    pingMs?: number;
}

export interface EffectResult {
    success: boolean;
    message: string;
}

export interface ColorResult {
    success: boolean;
    message: string;
}

export interface StreamResult {
    success: boolean;
    message: string;
}

// Device/Valise management types
export interface Device {
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
    customColor?: CustomColor;
}

export interface DeviceResult {
    success: boolean;
    message: string;
    device?: Device;
}

export interface DeviceGridSettings {
    columns: number;
    itemMinWidth: number;
    itemMaxWidth: number;
    gap: number;
    isResizable: boolean;
}
