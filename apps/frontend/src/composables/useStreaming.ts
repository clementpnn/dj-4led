// src/composables/useStreaming.ts
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { computed, onMounted, onUnmounted, ref } from 'vue';

// Types
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
}

interface StreamStatus {
    status: 'started' | 'stopped' | 'auto_stopped';
    message: string;
    stats?: StreamStats;
}

interface StreamingState {
    isStreaming: boolean;
    frameData: FrameData | null;
    spectrumData: number[];
    fps: number;
    streamStats: StreamStats;
    lastFrameTime: number;
    error: string | null;
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
    // État réactif principal
    const state = ref<StreamingState>({
        isStreaming: false,
        frameData: null,
        spectrumData: [],
        fps: 0,
        streamStats: {
            packets: 0,
            frames: 0,
            spectrum: 0,
        },
        lastFrameTime: 0,
        error: null,
    });

    // États pour compatibilité avec l'ancienne interface
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

    // FPS calculation
    let frameCount = 0;
    let lastFpsTime = Date.now();
    let fpsUpdateInterval: number | null = null;

    /**
     * Démarrer le streaming UDP
     */
    const startStream = async (): Promise<StreamResult> => {
        console.log('🚀 useStreaming: Starting UDP stream...');

        try {
            loading.value = true;
            state.value.error = null;

            const result = await invoke<string>('dj_start_stream');
            console.log('✅ useStreaming: Stream started:', result);

            state.value.isStreaming = true;
            state.value.streamStats = {
                packets: 0,
                frames: 0,
                spectrum: 0,
            };

            // Reset FPS counter
            frameCount = 0;
            lastFpsTime = Date.now();
            startFpsMonitoring();

            // Mise à jour des données legacy
            streamData.value.frames = [];
            streamData.value.spectrum = [];
            streamData.value.lastFrame = null;

            return {
                success: true,
                message: result,
            };
        } catch (error) {
            console.error('❌ useStreaming: Stream start error:', error);
            const errorMessage = error instanceof Error ? error.message : String(error);
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
     * Arrêter le streaming
     */
    const stopStream = async (): Promise<StreamResult> => {
        console.log('🛑 useStreaming: Stopping UDP stream...');

        try {
            loading.value = true;

            const result = await invoke<string>('dj_stop_stream');
            console.log('✅ useStreaming: Stream stopped:', result);

            state.value.isStreaming = false;
            stopFpsMonitoring();
            clearStreamData();

            return {
                success: true,
                message: result,
            };
        } catch (error) {
            console.error('❌ useStreaming: Stream stop error:', error);
            const errorMessage = error instanceof Error ? error.message : String(error);
            return {
                success: false,
                message: `Failed to stop stream: ${errorMessage}`,
            };
        } finally {
            loading.value = false;
        }
    };

    /**
     * Méthode legacy pour la compatibilité
     */
    const listenData = async (): Promise<StreamResult> => {
        console.log('🔄 useStreaming: listenData called (redirecting to startStream)');
        return await startStream();
    };

    /**
     * Effacer toutes les données de streaming
     */
    const clearStreamData = (): void => {
        console.log('🧹 useStreaming: Clearing stream data...');

        // Reset legacy data
        streamData.value = {
            frames: [],
            spectrum: [],
            lastFrame: null,
        };

        // Ne pas réinitialiser complètement l'état si on est en streaming
        if (!state.value.isStreaming) {
            state.value.frameData = null;
            state.value.spectrumData = [];
            state.value.lastFrameTime = 0;
        }
    };

    /**
     * Démarrer le monitoring FPS
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
            }
        }, 1000);
    };

    /**
     * Arrêter le monitoring FPS
     */
    const stopFpsMonitoring = (): void => {
        if (fpsUpdateInterval) {
            clearInterval(fpsUpdateInterval);
            fpsUpdateInterval = null;
        }
        state.value.fps = 0;
    };

    /**
     * Gérer les données de frame reçues du backend
     */
    const handleFrameData = (frameData: any): void => {
        // Validation des données de frame
        if (!frameData || typeof frameData.width !== 'number' || typeof frameData.height !== 'number') {
            console.warn('⚠️ useStreaming: Invalid frame data received:', frameData);
            return;
        }

        const processedFrameData: FrameData = {
            width: frameData.width,
            height: frameData.height,
            format: frameData.format || 1, // Default RGB
            data: Array.isArray(frameData.data) ? frameData.data : [],
            timestamp: Date.now(),
        };

        console.log(
            `🖼️ useStreaming: Frame received - ${processedFrameData.width}x${processedFrameData.height}, ${processedFrameData.data.length} bytes`
        );

        // Mise à jour de l'état principal
        state.value.frameData = processedFrameData;
        state.value.lastFrameTime = Date.now();
        state.value.streamStats.frames++;

        // Mise à jour des données legacy
        streamData.value.lastFrame = processedFrameData;
        streamData.value.frames.push(processedFrameData);

        // Limiter le nombre de frames stockées (garder seulement les 10 dernières)
        if (streamData.value.frames.length > 10) {
            streamData.value.frames = streamData.value.frames.slice(-10);
        }

        // Update FPS counter
        frameCount++;
    };

    /**
     * Gérer les données de frame compressées
     */
    const handleCompressedFrameData = (compressedData: number[]): void => {
        console.log('🗜️ useStreaming: Compressed frame received:', compressedData.length, 'bytes');
        state.value.streamStats.frames++;
        frameCount++;

        // TODO: Implémenter la décompression si nécessaire
        // Pour l'instant, on met à jour juste les compteurs
    };

    /**
     * Gérer les données de spectre audio reçues du backend
     */
    const handleSpectrumData = (spectrumData: number[]): void => {
        if (!Array.isArray(spectrumData) || spectrumData.length === 0) {
            return;
        }

        // Throttle spectrum updates pour éviter trop de re-renders
        const isDifferent = spectrumData.some((v, i) => Math.abs(v - (state.value.spectrumData[i] || 0)) > 0.05);

        if (isDifferent) {
            // Mise à jour de l'état principal
            state.value.spectrumData = [...spectrumData];
            state.value.streamStats.spectrum++;

            // Mise à jour des données legacy
            streamData.value.spectrum = [...spectrumData];

            if (state.value.streamStats.spectrum % 50 === 0) {
                console.log(
                    `🎵 useStreaming: Spectrum update #${state.value.streamStats.spectrum}, ${spectrumData.length} bands`
                );
            }
        }
    };

    /**
     * Gérer les mises à jour de statut du stream
     */
    const handleStreamStatus = (status: StreamStatus): void => {
        console.log('📊 useStreaming: Stream status update:', status);

        if (status.status === 'stopped' || status.status === 'auto_stopped') {
            state.value.isStreaming = false;
            stopFpsMonitoring();
        }

        if (status.stats) {
            state.value.streamStats = {
                ...state.value.streamStats,
                ...status.stats,
            };
        }
    };

    /**
     * Configurer les event listeners pour les données UDP
     */
    const setupEventListeners = async (): Promise<void> => {
        console.log('🎧 useStreaming: Setting up UDP event listeners...');

        try {
            // Écouter les données de frame (non compressées)
            unlistenFrame = await listen<any>('frame_data', event => {
                handleFrameData(event.payload);
            });

            // Écouter les données de frame compressées
            unlistenFrameCompressed = await listen<number[]>('frame_data_compressed', event => {
                handleCompressedFrameData(event.payload);
            });

            // Écouter les données de spectre
            unlistenSpectrum = await listen<number[]>('spectrum_data', event => {
                handleSpectrumData(event.payload);
            });

            // Écouter les mises à jour de statut du stream
            unlistenStreamStatus = await listen<StreamStatus>('stream_status', event => {
                handleStreamStatus(event.payload);
            });

            console.log('✅ useStreaming: UDP event listeners ready');
        } catch (error) {
            console.error('❌ useStreaming: Error setting up event listeners:', error);
            state.value.error = `Failed to setup event listeners: ${error}`;
        }
    };

    /**
     * Nettoyer les event listeners
     */
    const cleanup = (): void => {
        console.log('🧹 useStreaming: Cleaning up...');

        const listeners = [
            { ref: unlistenFrame, name: 'frame' },
            { ref: unlistenFrameCompressed, name: 'frameCompressed' },
            { ref: unlistenSpectrum, name: 'spectrum' },
            { ref: unlistenStreamStatus, name: 'streamStatus' },
        ];

        listeners.forEach(({ ref, name }) => {
            if (ref) {
                ref();
                console.log(`✅ useStreaming: Cleaned up ${name} listener`);
            }
        });

        // Reset listener references
        unlistenFrame = null;
        unlistenFrameCompressed = null;
        unlistenSpectrum = null;
        unlistenStreamStatus = null;

        // Arrêter le monitoring
        stopFpsMonitoring();
    };

    /**
     * Réinitialiser l'état aux valeurs initiales
     */
    const reset = (): void => {
        console.log('🔄 useStreaming: Resetting state...');

        // Arrêter le monitoring
        stopFpsMonitoring();

        state.value = {
            isStreaming: false,
            frameData: null,
            spectrumData: [],
            fps: 0,
            streamStats: {
                packets: 0,
                frames: 0,
                spectrum: 0,
            },
            lastFrameTime: 0,
            error: null,
        };

        // Reset legacy data
        streamData.value = {
            frames: [],
            spectrum: [],
            lastFrame: null,
        };

        frameCount = 0;
        lastFpsTime = Date.now();
        loading.value = false;
    };

    /**
     * Obtenir les informations de connexion du serveur
     */
    const getServerInfo = async (): Promise<string> => {
        try {
            const result = await invoke<string>('dj_get_server_info');
            return result;
        } catch (error) {
            console.error('❌ useStreaming: Error getting server info:', error);
            return `Error getting server info: ${error}`;
        }
    };

    /**
     * Vérifier si le streaming est en bonne santé (reçoit des données récemment)
     */
    const isStreamHealthy = (): boolean => {
        if (!state.value.isStreaming) return false;
        if (state.value.lastFrameTime === 0) return true; // Vient de commencer

        const timeSinceLastFrame = Date.now() - state.value.lastFrameTime;
        return timeSinceLastFrame < 5000; // Considéré comme malsain si pas de données depuis 5s
    };

    // Propriétés computed pour une meilleure réactivité
    const isStreaming = computed(() => state.value.isStreaming);
    const frameData = computed(() => state.value.frameData);
    const spectrumData = computed(() => state.value.spectrumData);
    const fps = computed(() => state.value.fps);
    const streamStats = computed(() => state.value.streamStats);
    const error = computed(() => state.value.error);
    const lastFrameTime = computed(() => state.value.lastFrameTime);

    // Configuration au montage
    onMounted(() => {
        console.log('🚀 useStreaming: Component mounted, setting up UDP listeners...');
        setupEventListeners();
    });

    // Nettoyage au démontage
    onUnmounted(() => {
        console.log('💀 useStreaming: Component unmounting...');
        cleanup();
        if (state.value.isStreaming) {
            console.log('🛑 useStreaming: Auto-stopping stream on unmount...');
            stopStream();
        }
    });

    return {
        // État réactif principal
        state,

        // Legacy compatibility - états réactifs
        loading,
        streamData,

        // Getters computed
        isStreaming,
        frameData,
        spectrumData,
        fps,
        streamStats,
        error,
        lastFrameTime,

        // Actions principales
        startStream,
        stopStream,
        listenData, // Legacy method
        clearStreamData,
        reset,
        getServerInfo,
        isStreamHealthy,

        // Utilitaires
        setupEventListeners,
        cleanup,
    };
}
