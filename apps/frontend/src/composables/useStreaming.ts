import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { computed, onMounted, onUnmounted, ref } from "vue";

interface FrameData {
  width: number;
  height: number;
  format: number;
  data: number[];
  timestamp: number;
}

interface StreamStats {
  packets: number;
  frames: number;
  spectrum: number;
  duration?: number;
  bytesReceived?: number;
  packetsLost?: number;
}

interface StreamStatus {
  status: "started" | "stopped" | "auto_stopped" | "error";
  message: string;
  stats?: StreamStats;
  error?: string;
}

interface StreamingState {
  isStreaming: boolean;
  frameData: FrameData | null;
  spectrumData: number[];
  fps: number;
  streamStats: StreamStats;
  lastFrameTime: number;
  lastSpectrumTime: number;
  error: string | null;
  connectionQuality: number;
}

interface StreamResult {
  success: boolean;
  message: string;
}

interface StreamData {
  frames: any[];
  spectrum: number[];
  lastFrame: any | null;
}

export function useStreaming() {
  const state = ref<StreamingState>({
    isStreaming: false,
    frameData: null,
    spectrumData: [],
    fps: 0,
    streamStats: {
      packets: 0,
      frames: 0,
      spectrum: 0,
      bytesReceived: 0,
      packetsLost: 0,
    },
    lastFrameTime: 0,
    lastSpectrumTime: 0,
    error: null,
    connectionQuality: 0,
  });

  // Legacy compatibility
  const loading = ref(false);
  const streamData = ref<StreamData>({
    frames: [],
    spectrum: [],
    lastFrame: null,
  });

  // Event listeners
  let unlistenFrame: UnlistenFn | null = null;
  let unlistenFrameCompressed: UnlistenFn | null = null;
  let unlistenSpectrum: UnlistenFn | null = null;
  let unlistenStreamStatus: UnlistenFn | null = null;

  // FPS and quality monitoring
  let frameCount = 0;
  let lastFpsTime = Date.now();
  let fpsUpdateInterval: number | null = null;
  let qualityCheckInterval: number | null = null;
  let lastDataReceived = Date.now();

  // Spectrum smoothing
  let previousSpectrum: number[] = [];
  const spectrumSmoothingFactor = 0.7;

  /**
   * Enhanced stream start with better error handling
   */
  const startStream = async (): Promise<StreamResult> => {
    console.log("🚀 useStreaming: Starting enhanced UDP stream...");

    try {
      loading.value = true;
      state.value.error = null;

      // Pre-flight check
      if (state.value.isStreaming) {
        console.log("⚠️ useStreaming: Stream already active");
        return {
          success: false,
          message: "Stream is already active",
        };
      }

      const result = await invoke<string>("dj_start_stream");
      console.log("✅ useStreaming: Stream started:", result);

      // Initialize state
      state.value.isStreaming = true;
      state.value.streamStats = {
        packets: 0,
        frames: 0,
        spectrum: 0,
        bytesReceived: 0,
        packetsLost: 0,
      };
      state.value.connectionQuality = 50; // Start with medium quality

      // Reset counters
      frameCount = 0;
      lastFpsTime = Date.now();
      lastDataReceived = Date.now();

      // Start monitoring
      startFpsMonitoring();
      startQualityMonitoring();

      // Reset legacy data
      streamData.value.frames = [];
      streamData.value.spectrum = [];
      streamData.value.lastFrame = null;

      return {
        success: true,
        message: result,
      };
    } catch (error) {
      console.error("❌ useStreaming: Stream start error:", error);
      const errorMessage =
        error instanceof Error ? error.message : String(error);
      state.value.error = errorMessage;

      return {
        success: false,
        message: `Failed to start stream: ${errorMessage}`,
      };
    } finally {
      loading.value = false;
    }
  };

  /**
   * Enhanced stream stop with cleanup
   */
  const stopStream = async (): Promise<StreamResult> => {
    console.log("🛑 useStreaming: Stopping enhanced UDP stream...");

    try {
      loading.value = true;

      const result = await invoke<string>("dj_stop_stream");
      console.log("✅ useStreaming: Stream stopped:", result);

      // Clean state
      state.value.isStreaming = false;
      stopFpsMonitoring();
      stopQualityMonitoring();
      clearStreamData();

      return {
        success: true,
        message: result,
      };
    } catch (error) {
      console.error("❌ useStreaming: Stream stop error:", error);
      const errorMessage =
        error instanceof Error ? error.message : String(error);
      return {
        success: false,
        message: `Failed to stop stream: ${errorMessage}`,
      };
    } finally {
      loading.value = false;
    }
  };

  /**
   * Clear streaming error
   */
  const clearError = (): void => {
    state.value.error = null;
  };

  /**
   * Legacy compatibility method
   */
  const listenData = async (): Promise<StreamResult> => {
    console.log(
      "🔄 useStreaming: listenData called (redirecting to startStream)",
    );
    return await startStream();
  };

  /**
   * Enhanced data clearing
   */
  const clearStreamData = (): void => {
    console.log("🧹 useStreaming: Clearing stream data...");

    // Reset legacy data
    streamData.value = {
      frames: [],
      spectrum: [],
      lastFrame: null,
    };

    // Reset spectrum smoothing
    previousSpectrum = [];

    // Clean state if not streaming
    if (!state.value.isStreaming) {
      state.value.frameData = null;
      state.value.spectrumData = [];
      state.value.lastFrameTime = 0;
      state.value.lastSpectrumTime = 0;
      state.value.connectionQuality = 0;
    }
  };

  /**
   * Enhanced FPS monitoring
   */
  const startFpsMonitoring = (): void => {
    if (fpsUpdateInterval) clearInterval(fpsUpdateInterval);

    fpsUpdateInterval = window.setInterval(() => {
      const now = Date.now();
      const elapsed = now - lastFpsTime;

      if (elapsed >= 1000) {
        const newFps = Math.round((frameCount * 1000) / elapsed);
        state.value.fps = newFps;
        frameCount = 0;
        lastFpsTime = now;

        // Log significant FPS changes
        if (state.value.isStreaming && (newFps === 0 || newFps < 5)) {
          console.warn(`⚠️ useStreaming: Low FPS detected: ${newFps}`);
        }
      }
    }, 1000);
  };

  /**
   * Connection quality monitoring
   */
  const startQualityMonitoring = (): void => {
    if (qualityCheckInterval) clearInterval(qualityCheckInterval);

    qualityCheckInterval = window.setInterval(() => {
      const now = Date.now();
      const timeSinceLastData = now - lastDataReceived;
      const currentFps = state.value.fps;

      // Calculate quality based on data freshness and FPS
      let quality = 100;

      if (timeSinceLastData > 5000) {
        quality = 0; // No data for 5+ seconds
      } else if (timeSinceLastData > 2000) {
        quality = 25; // Stale data
      } else if (currentFps < 5) {
        quality = 30; // Very low FPS
      } else if (currentFps < 15) {
        quality = 60; // Low FPS
      } else if (currentFps < 25) {
        quality = 80; // Good FPS
      }

      // Smooth quality changes
      const smoothingFactor = 0.8;
      state.value.connectionQuality = Math.round(
        state.value.connectionQuality * smoothingFactor +
          quality * (1 - smoothingFactor),
      );
    }, 2000);
  };

  /**
   * Stop monitoring intervals
   */
  const stopFpsMonitoring = (): void => {
    if (fpsUpdateInterval) {
      clearInterval(fpsUpdateInterval);
      fpsUpdateInterval = null;
    }
    state.value.fps = 0;
  };

  const stopQualityMonitoring = (): void => {
    if (qualityCheckInterval) {
      clearInterval(qualityCheckInterval);
      qualityCheckInterval = null;
    }
  };

  /**
   * Enhanced frame data handling
   */
  const handleFrameData = (frameData: any): void => {
    const now = Date.now();
    lastDataReceived = now;

    // Validation
    if (
      !frameData ||
      typeof frameData.width !== "number" ||
      typeof frameData.height !== "number"
    ) {
      console.warn("⚠️ useStreaming: Invalid frame data received:", frameData);
      return;
    }

    const processedFrameData: FrameData = {
      width: frameData.width,
      height: frameData.height,
      format: frameData.format || 1,
      data: Array.isArray(frameData.data) ? frameData.data : [],
      timestamp: now,
    };

    console.log(
      `🖼️ useStreaming: Frame ${state.value.streamStats.frames + 1} - ${processedFrameData.width}x${
        processedFrameData.height
      }, ${processedFrameData.data.length}B`,
    );

    // Update state
    state.value.frameData = processedFrameData;
    state.value.lastFrameTime = now;
    state.value.streamStats.frames++;
    state.value.streamStats.bytesReceived =
      (state.value.streamStats.bytesReceived || 0) +
      processedFrameData.data.length;

    // Update legacy data with size limit
    streamData.value.lastFrame = processedFrameData;
    streamData.value.frames.push(processedFrameData);
    if (streamData.value.frames.length > 10) {
      streamData.value.frames = streamData.value.frames.slice(-10);
    }

    // Update FPS counter
    frameCount++;
  };

  /**
   * Enhanced compressed frame handling
   */
  const handleCompressedFrameData = (compressedData: number[]): void => {
    const now = Date.now();
    lastDataReceived = now;

    console.log(
      `🗜️ useStreaming: Compressed frame ${state.value.streamStats.frames + 1} - ${compressedData.length}B`,
    );

    state.value.streamStats.frames++;
    state.value.streamStats.bytesReceived =
      (state.value.streamStats.bytesReceived || 0) + compressedData.length;
    frameCount++;

    // TODO: Implement decompression if needed
  };

  /**
   * Enhanced spectrum data handling with smoothing
   */
  const handleSpectrumData = (spectrumData: number[]): void => {
    const now = Date.now();
    lastDataReceived = now;

    if (!Array.isArray(spectrumData) || spectrumData.length === 0) {
      return;
    }

    // Apply smoothing to reduce jitter
    let smoothedSpectrum: number[];
    if (previousSpectrum.length === spectrumData.length) {
      smoothedSpectrum = spectrumData.map((value, index) => {
        const prevValue = previousSpectrum[index] || 0;
        return (
          prevValue * spectrumSmoothingFactor +
          value * (1 - spectrumSmoothingFactor)
        );
      });
    } else {
      smoothedSpectrum = [...spectrumData];
    }

    // Update state
    state.value.spectrumData = smoothedSpectrum;
    state.value.lastSpectrumTime = now;
    state.value.streamStats.spectrum++;

    // Update legacy data
    streamData.value.spectrum = smoothedSpectrum;

    // Store for next smoothing iteration
    previousSpectrum = smoothedSpectrum;

    // Periodic logging (every 50th update)
    if (state.value.streamStats.spectrum % 50 === 0) {
      console.log(
        `🎵 useStreaming: Spectrum update #${state.value.streamStats.spectrum}, ${
          spectrumData.length
        } bands, peak: ${Math.max(...smoothedSpectrum).toFixed(2)}`,
      );
    }
  };

  /**
   * Enhanced stream status handling
   */
  const handleStreamStatus = (status: StreamStatus): void => {
    console.log("📊 useStreaming: Stream status update:", status);

    if (status.status === "stopped" || status.status === "auto_stopped") {
      state.value.isStreaming = false;
      stopFpsMonitoring();
      stopQualityMonitoring();
    }

    if (status.status === "error") {
      state.value.error = status.error || status.message;
    }

    if (status.stats) {
      state.value.streamStats = {
        ...state.value.streamStats,
        ...status.stats,
      };
    }
  };

  /**
   * Enhanced event listeners setup
   */
  const setupEventListeners = async (): Promise<void> => {
    console.log("🎧 useStreaming: Setting up enhanced UDP event listeners...");

    try {
      // Frame data listener with error handling
      unlistenFrame = await listen<any>("frame_data", (event) => {
        try {
          handleFrameData(event.payload);
        } catch (error) {
          console.error("❌ useStreaming: Error handling frame data:", error);
          state.value.streamStats.packetsLost =
            (state.value.streamStats.packetsLost || 0) + 1;
        }
      });

      // Compressed frame data listener
      unlistenFrameCompressed = await listen<number[]>(
        "frame_data_compressed",
        (event) => {
          try {
            handleCompressedFrameData(event.payload);
          } catch (error) {
            console.error(
              "❌ useStreaming: Error handling compressed frame:",
              error,
            );
            state.value.streamStats.packetsLost =
              (state.value.streamStats.packetsLost || 0) + 1;
          }
        },
      );

      // Spectrum data listener with error handling
      unlistenSpectrum = await listen<number[]>("spectrum_data", (event) => {
        try {
          handleSpectrumData(event.payload);
        } catch (error) {
          console.error(
            "❌ useStreaming: Error handling spectrum data:",
            error,
          );
          state.value.streamStats.packetsLost =
            (state.value.streamStats.packetsLost || 0) + 1;
        }
      });

      // Stream status listener
      unlistenStreamStatus = await listen<StreamStatus>(
        "stream_status",
        (event) => {
          try {
            handleStreamStatus(event.payload);
          } catch (error) {
            console.error(
              "❌ useStreaming: Error handling stream status:",
              error,
            );
          }
        },
      );

      console.log("✅ useStreaming: Enhanced UDP event listeners ready");
    } catch (error) {
      console.error(
        "❌ useStreaming: Error setting up event listeners:",
        error,
      );
      state.value.error = `Failed to setup event listeners: ${error}`;
    }
  };

  /**
   * Enhanced cleanup with better error handling
   */
  const cleanup = (): void => {
    console.log("🧹 useStreaming: Enhanced cleanup...");

    const listeners = [
      { ref: unlistenFrame, name: "frame" },
      { ref: unlistenFrameCompressed, name: "frameCompressed" },
      { ref: unlistenSpectrum, name: "spectrum" },
      { ref: unlistenStreamStatus, name: "streamStatus" },
    ];

    listeners.forEach(({ ref, name }) => {
      if (ref) {
        try {
          ref();
          console.log(`✅ useStreaming: Cleaned up ${name} listener`);
        } catch (error) {
          console.error(
            `❌ useStreaming: Error cleaning up ${name} listener:`,
            error,
          );
        }
      }
    });

    // Reset listener references
    unlistenFrame = null;
    unlistenFrameCompressed = null;
    unlistenSpectrum = null;
    unlistenStreamStatus = null;

    // Stop monitoring
    stopFpsMonitoring();
    stopQualityMonitoring();
  };

  /**
   * Enhanced state reset
   */
  const reset = (): void => {
    console.log("🔄 useStreaming: Enhanced state reset...");

    // Stop monitoring
    stopFpsMonitoring();
    stopQualityMonitoring();

    // Reset state
    state.value = {
      isStreaming: false,
      frameData: null,
      spectrumData: [],
      fps: 0,
      streamStats: {
        packets: 0,
        frames: 0,
        spectrum: 0,
        bytesReceived: 0,
        packetsLost: 0,
      },
      lastFrameTime: 0,
      lastSpectrumTime: 0,
      error: null,
      connectionQuality: 0,
    };

    // Reset legacy data
    streamData.value = {
      frames: [],
      spectrum: [],
      lastFrame: null,
    };

    // Reset counters and smoothing
    frameCount = 0;
    lastFpsTime = Date.now();
    lastDataReceived = Date.now();
    previousSpectrum = [];
    loading.value = false;
  };

  /**
   * Get server information
   */
  const getServerInfo = async (): Promise<string> => {
    try {
      const result = await invoke<string>("dj_get_server_info");
      return result;
    } catch (error) {
      console.error("❌ useStreaming: Error getting server info:", error);
      return `Error getting server info: ${error}`;
    }
  };

  /**
   * Enhanced stream health check
   */
  const isStreamHealthy = (): boolean => {
    if (!state.value.isStreaming) return false;

    const now = Date.now();
    const timeSinceLastFrame = now - state.value.lastFrameTime;
    const timeSinceLastSpectrum = now - state.value.lastSpectrumTime;

    // Consider healthy if:
    // 1. Just started (no data yet)
    // 2. Recent frame data (within 3 seconds)
    // 3. Recent spectrum data (within 2 seconds)
    // 4. Good connection quality (> 30%)

    if (state.value.lastFrameTime === 0 && state.value.lastSpectrumTime === 0) {
      return true; // Just started
    }

    const hasRecentFrames = timeSinceLastFrame < 3000;
    const hasRecentSpectrum = timeSinceLastSpectrum < 2000;
    const goodQuality = state.value.connectionQuality > 30;

    return (hasRecentFrames || hasRecentSpectrum) && goodQuality;
  };

  /**
   * Get detailed stream statistics
   */
  const getStreamStatistics = () => {
    const now = Date.now();
    return {
      ...state.value.streamStats,
      fps: state.value.fps,
      connectionQuality: state.value.connectionQuality,
      timeSinceLastFrame: state.value.lastFrameTime
        ? now - state.value.lastFrameTime
        : 0,
      timeSinceLastSpectrum: state.value.lastSpectrumTime
        ? now - state.value.lastSpectrumTime
        : 0,
      isHealthy: isStreamHealthy(),
      dataRate: state.value.streamStats.bytesReceived
        ? Math.round((state.value.streamStats.bytesReceived || 0) / 1024)
        : 0, // KB
    };
  };

  // Enhanced computed properties
  const isStreaming = computed(() => state.value.isStreaming);
  const frameData = computed(() => state.value.frameData);
  const spectrumData = computed(() => state.value.spectrumData);
  const fps = computed(() => state.value.fps);
  const streamStats = computed(() => state.value.streamStats);
  const error = computed(() => state.value.error);
  const lastFrameTime = computed(() => state.value.lastFrameTime);
  const connectionQuality = computed(() => state.value.connectionQuality);

  // Lifecycle management
  onMounted(() => {
    console.log(
      "🚀 useStreaming: Enhanced component mounted, setting up UDP listeners...",
    );
    setupEventListeners();
  });

  onUnmounted(() => {
    console.log("💀 useStreaming: Enhanced component unmounting...");
    cleanup();
    if (state.value.isStreaming) {
      console.log("🛑 useStreaming: Auto-stopping stream on unmount...");
      stopStream();
    }
  });

  return {
    // Enhanced reactive state
    state,

    // Legacy compatibility
    loading,
    streamData,

    // Enhanced computed getters
    isStreaming,
    frameData,
    spectrumData,
    fps,
    streamStats,
    error,
    lastFrameTime,
    connectionQuality,

    // Enhanced actions
    startStream,
    stopStream,
    clearError,
    listenData, // Legacy method
    clearStreamData,
    reset,
    getServerInfo,
    isStreamHealthy,
    getStreamStatistics,

    // Enhanced utilities
    setupEventListeners,
    cleanup,
  };
}
