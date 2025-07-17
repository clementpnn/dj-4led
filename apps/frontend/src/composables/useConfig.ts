// src/composables/useConfig.ts
import { invoke } from "@tauri-apps/api/core";
import { computed, ref, watch } from "vue";

export interface ServerConfig {
  ip: string;
  port: number;
}

export interface AudioConfig {
  deviceId: string;
  deviceName?: string;
  gain: number;
  sampleRate?: number;
  bufferSize?: number;
}

export interface LedController {
  ip: string;
  port: number;
  enabled?: boolean;
}

export interface DisplayConfig {
  mode: "auto" | "grid" | "strip" | "circular";
  scale: number;
  showFps?: boolean;
  interpolation?: boolean;
}

export interface AppConfig {
  server: ServerConfig;
  audio: AudioConfig;
  controllers: LedController[];
  display: DisplayConfig;
}

// Default configuration
const defaultConfig: AppConfig = {
  server: {
    ip: "127.0.0.1",
    port: 8081,
  },
  audio: {
    deviceId: "",
    gain: 2.0,
    sampleRate: 48000,
    bufferSize: 32,
  },
  controllers: [
    { ip: "192.168.1.45", port: 6454, enabled: true },
    { ip: "192.168.1.46", port: 6454, enabled: true },
    { ip: "192.168.1.47", port: 6454, enabled: true },
    { ip: "192.168.1.48", port: 6454, enabled: true },
  ],
  display: {
    mode: "auto",
    scale: 1.0,
    showFps: true,
    interpolation: true,
  },
};

// Global state
const config = ref<AppConfig>(loadConfigFromStorage());
const configLoaded = ref(false);

// Save to localStorage whenever config changes
watch(
  config,
  (newConfig) => {
    saveConfigToStorage(newConfig);
  },
  { deep: true },
);

// Helper functions
function loadConfigFromStorage(): AppConfig {
  const stored = localStorage.getItem("dj4led-config");
  if (stored) {
    try {
      const parsed = JSON.parse(stored);
      // Merge with defaults to ensure all fields exist
      return deepMerge(defaultConfig, parsed);
    } catch (error) {
      console.error("Failed to parse stored config:", error);
    }
  }
  return { ...defaultConfig };
}

function saveConfigToStorage(config: AppConfig): void {
  try {
    localStorage.setItem("dj4led-config", JSON.stringify(config));
  } catch (error) {
    console.error("Failed to save config:", error);
  }
}

function deepMerge(target: any, source: any): any {
  const output = { ...target };
  if (isObject(target) && isObject(source)) {
    Object.keys(source).forEach((key) => {
      if (isObject(source[key])) {
        if (!(key in target)) {
          Object.assign(output, { [key]: source[key] });
        } else {
          output[key] = deepMerge(target[key], source[key]);
        }
      } else {
        Object.assign(output, { [key]: source[key] });
      }
    });
  }
  return output;
}

function isObject(item: any): boolean {
  return item && typeof item === "object" && !Array.isArray(item);
}

// Composable
export function useConfig() {
  // Computed values
  const serverAddress = computed(
    () => `${config.value.server.ip}:${config.value.server.port}`,
  );

  const totalLeds = computed(() =>
    config.value.controllers.reduce(
      (total, controller) =>
        total +
        (controller.enabled !== false
          ? controller.width * controller.height
          : 0),
      0,
    ),
  );

  const activeControllers = computed(() =>
    config.value.controllers.filter((c) => c.enabled !== false),
  );

  const displayDimensions = computed(() => {
    const controllers = activeControllers.value;
    if (controllers.length === 0) return { width: 0, height: 0 };

    // Calculate total dimensions based on layout
    if (config.value.display.mode === "grid") {
      // Arrange controllers in a grid
      const cols = Math.ceil(Math.sqrt(controllers.length));
      const rows = Math.ceil(controllers.length / cols);
      const maxWidth = Math.max(...controllers.map((c) => c.width));
      const maxHeight = Math.max(...controllers.map((c) => c.height));
      return {
        width: cols * maxWidth,
        height: rows * maxHeight,
      };
    } else if (config.value.display.mode === "strip") {
      // Horizontal strip
      const totalWidth = controllers.reduce((sum, c) => sum + c.width, 0);
      const maxHeight = Math.max(...controllers.map((c) => c.height));
      return {
        width: totalWidth,
        height: maxHeight,
      };
    } else {
      // Auto mode - assumes controllers are side by side
      const totalWidth = controllers.reduce((sum, c) => sum + c.width, 0);
      const maxHeight = Math.max(...controllers.map((c) => c.height));
      return {
        width: totalWidth,
        height: maxHeight,
      };
    }
  });

  // Methods
  const updateConfig = (newConfig: Partial<AppConfig>) => {
    config.value = deepMerge(config.value, newConfig);
  };

  const resetConfig = () => {
    config.value = { ...defaultConfig };
  };

  const applyServerConfig = async () => {
    try {
      // This would update the Rust backend with new server config
      await invoke("dj_update_server_config", {
        ip: config.value.server.ip,
        port: config.value.server.port,
      });
      return true;
    } catch (error) {
      console.error("Failed to apply server config:", error);
      return false;
    }
  };

  const applyAudioConfig = async () => {
    try {
      // This would update the audio configuration
      await invoke("dj_update_audio_config", {
        deviceId: config.value.audio.deviceId,
        gain: config.value.audio.gain,
        sampleRate: config.value.audio.sampleRate,
        bufferSize: config.value.audio.bufferSize,
      });
      return true;
    } catch (error) {
      console.error("Failed to apply audio config:", error);
      return false;
    }
  };

  const applyControllerConfig = async () => {
    try {
      // Update the LED controller configuration
      const controllers = config.value.controllers.map((c) => ({
        address: c.ip,
        port: c.port,
        enabled: c.enabled !== false,
      }));

      await invoke("dj_update_controllers", {
        controllers,
        serverIp: config.value.server.ip,
        serverPort: config.value.server.port,
      });
      return true;
    } catch (error) {
      console.error("Failed to apply controller config:", error);
      return false;
    }
  };

  const applyAllConfigs = async () => {
    const results = await Promise.all([
      applyServerConfig(),
      applyAudioConfig(),
      applyControllerConfig(),
    ]);
    return results.every((r) => r === true);
  };

  const exportConfig = () => {
    const dataStr = JSON.stringify(config.value, null, 2);
    const blob = new Blob([dataStr], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = `dj4led-config-${new Date().toISOString().split("T")[0]}.json`;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
  };

  const importConfig = (file: File): Promise<void> => {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = (e) => {
        try {
          const imported = JSON.parse(e.target?.result as string);
          config.value = deepMerge(defaultConfig, imported);
          resolve();
        } catch (error) {
          reject(error);
        }
      };
      reader.onerror = reject;
      reader.readAsText(file);
    });
  };

  return {
    // State
    config: computed(() => config.value),
    configLoaded: computed(() => configLoaded.value),

    // Computed
    serverAddress,
    totalLeds,
    activeControllers,
    displayDimensions,

    // Methods
    updateConfig,
    resetConfig,
    applyServerConfig,
    applyAudioConfig,
    applyControllerConfig,
    applyAllConfigs,
    exportConfig,
    importConfig,
  };
}
