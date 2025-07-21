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
  data: number[];
  size?: number;
  timestamp?: number;
}

export interface StreamData {
  frames: FrameData[];
  spectrum: number[];
  lastFrame: FrameData | null;
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

// LED Preview specific types
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

export interface LedStreamResult {
  success: boolean;
  message: string;
}
