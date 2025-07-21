import { Ref } from "vue";

export interface CustomColor {
  r: number;
  g: number;
  b: number;
}

export interface FrameData {
  width: number;
  height: number;
  format: number;
  data: number[];
  size?: number;
  timestamp?: number;
}

export interface StreamData {
  frames: FrameData[];
  spectrum: number[];
  lastFrame: FrameData | undefined;
}

export interface LogEntry {
  time: string;
  message: string;
  type: "info" | "success" | "error" | "warning";
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
  key: "r" | "g" | "b";
  name: string;
  emoji: string;
}

export interface StreamStats {
  packets: number;
  frames: number;
  spectrum: number;
  duration?: number;
}

export interface StreamStatus {
  status: "stopped" | "auto_stopped" | "running";
  message: string;
  stats?: StreamStats;
}

export interface LedPreviewState {
  isStreaming: boolean;
  frameData: FrameData | null;
  spectrumData: number[];
  fps: number;
  streamStats: StreamStats;
  lastFrameTime: number;
  error: string | null;
}

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

export interface LedStreamResult {
  success: boolean;
  message: string;
}

// Event payloads for Tauri events
export interface TauriFrameDataEvent {
  width: number;
  height: number;
  format: number;
  data: number[];
}

export interface TauriSpectrumDataEvent extends Array<number> {}

export interface TauriStreamStatusEvent {
  status: string;
  message: string;
  stats?: {
    packets: number;
    frames: number;
    spectrum: number;
    duration?: number;
  };
}

// Format enum for better type safety
export enum FrameFormat {
  RGB = 1,
  RGBA = 2,
  HSV = 3,
  HSL = 4,
}

// Utility types
export type LogType = "info" | "success" | "error" | "warning";

export type ColorChannelKey = "r" | "g" | "b";

export type StreamStatusType = "stopped" | "auto_stopped" | "running";

// Advanced types for better composition
export interface StreamMetadata {
  startTime: number;
  lastUpdate: number;
  totalPackets: number;
  averageFps: number;
  isHealthy: boolean;
}

export interface LedMatrixConfig {
  width: number;
  height: number;
  pixelFormat: FrameFormat;
  refreshRate: number;
}

// Error types for better error handling
export interface DJError {
  code: string;
  message: string;
  details?: any;
}

export interface NetworkError extends DJError {
  code: "NETWORK_ERROR";
  networkCode?: string;
}

export interface ParseError extends DJError {
  code: "PARSE_ERROR";
  rawData?: any;
}

export interface StreamError extends DJError {
  code: "STREAM_ERROR";
  streamType?: "frame" | "spectrum";
}

// Composable return types for better intellisense
export interface UseConnectionReturn {
  isConnected: Ref<boolean>;
  loading: Ref<boolean>;
  pingMs: Ref<number>;
  connect: () => Promise<ConnectionResult>;
  disconnect: () => Promise<ConnectionResult>;
  ping: () => Promise<ConnectionResult>;
}

export interface UseLedPreviewReturn {
  state: Ref<LedPreviewState>;
  isStreaming: boolean;
  frameData: FrameData | null;
  spectrumData: number[];
  fps: number;
  streamStats: StreamStats;
  error: string | null;
  lastFrameTime: number;
  startStream: () => Promise<LedStreamResult>;
  stopStream: () => Promise<LedStreamResult>;
  reset: () => void;
  getServerInfo: () => Promise<string>;
  isStreamHealthy: () => boolean;
  setupEventListeners: () => Promise<void>;
  cleanup: () => void;
}
