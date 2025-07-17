// src/composables/useStreaming.ts
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { onMounted, ref } from "vue";
import { useConfig } from "./useConfig";

interface FrameData {
  width: number;
  height: number;
  format: number;
  size: number;
  timestamp: number;
}

interface StreamData {
  frames: FrameData[];
  spectrum: number[];
  lastFrame: FrameData | null;
}

interface StreamResult {
  success: boolean;
  message: string;
}

export function useStreaming() {
  const fps = ref<number>(0);
  const loading = ref<boolean>(false);
  const isStreaming = ref<boolean>(false);
  const streamData = ref<StreamData>({
    frames: [],
    spectrum: [],
    lastFrame: null,
  });
  const { config } = useConfig();

  const listenData = async (): Promise<StreamResult> => {
    loading.value = true;
    isStreaming.value = true;
    const startTime = performance.now();
    try {
      const result = await invoke<string>("dj_listen_data", {
        serverIp: config.value.server.ip,
        serverPort: config.value.server.port,
      });
      const endTime = performance.now();
      const duration = (endTime - startTime) / 1000;

      // Extraire le nombre de frames du résultat
      const frameMatch = result.match(/(\d+) frames/);
      if (frameMatch) {
        const frameCount = parseInt(frameMatch[1]);
        fps.value = Math.round(frameCount / duration);
      } else {
        fps.value = 0;
      }

      return { success: true, message: result };
    } catch (error) {
      fps.value = 0;
      isStreaming.value = false;
      return { success: false, message: `❌ Stream error: ${error}` };
    } finally {
      loading.value = false;
    }
  };

  const clearStreamData = (): void => {
    fps.value = 0;
    isStreaming.value = false;
    streamData.value = {
      frames: [],
      spectrum: [],
      lastFrame: null,
    };
  };

  // Setup event listeners
  const setupEventListeners = async (): Promise<void> => {
    // Listen for frame data
    await listen("frame_data", (event) => {
      const data = event.payload as Uint8Array;
      if (data.length >= 5) {
        const frameData: FrameData = {
          width: (data[1] << 8) | data[0],
          height: (data[3] << 8) | data[2],
          format: data[4],
          size: data.length - 5,
          timestamp: Date.now(),
        };

        streamData.value.frames.push(frameData);
        streamData.value.lastFrame = frameData;

        // Keep only last 60 frames (for FPS calculation)
        if (streamData.value.frames.length > 60) {
          streamData.value.frames.shift();
        }
      }
    });

    // Listen for spectrum data
    await listen("spectrum_data", (event) => {
      const data = event.payload as number[];
      streamData.value.spectrum = data;
    });
  };

  onMounted(() => {
    setupEventListeners();
  });

  return {
    fps,
    loading,
    isStreaming,
    streamData,
    listenData,
    clearStreamData,
  };
}
