import { useCallback, useEffect, useRef, useState } from "react";
import "./App.css";

interface Effect {
  id: number;
  name: string;
  icon: string;
}

interface ColorMode {
  id: string;
  name: string;
  icon: string;
}

interface FrameMessage {
  type: "frame";
  data: number[];
}

interface CompressedFrameMessage {
  type: "compressedframe";
  data: number[];
  width: number;
  height: number;
}

interface SpectrumMessage {
  type: "spectrum";
  data: number[];
}

type WebSocketMessage = FrameMessage | CompressedFrameMessage | SpectrumMessage;

// Décompresseur pour les frames compressées
class FrameDecompressor {
  async decompress(data: Uint8Array): Promise<Uint8Array> {
    try {
      // Utiliser l'API Compression Streams si disponible
      if ("DecompressionStream" in (window as any)) {
        const ds = new (window as any).DecompressionStream("gzip");
        const writer = ds.writable.getWriter();
        const reader = ds.readable.getReader();

        writer.write(data);
        writer.close();

        const chunks: Uint8Array[] = [];
        let result: any;
        while (!(result = await reader.read()).done) {
          chunks.push(result.value);
        }

        // Combiner tous les chunks
        const totalLength = chunks.reduce(
          (acc, chunk) => acc + chunk.length,
          0,
        );
        const combined = new Uint8Array(totalLength);
        let offset = 0;
        for (const chunk of chunks) {
          combined.set(chunk, offset);
          offset += chunk.length;
        }

        return combined;
      }
    } catch (error) {
      console.error("Decompression failed:", error);
    }

    // Fallback: retourner les données telles quelles
    return data;
  }
}

// Optimisation du rendu avec requestAnimationFrame
class FrameRenderer {
  private canvas: HTMLCanvasElement | null = null;
  private ctx: CanvasRenderingContext2D | null = null;
  private imageData: ImageData | null = null;
  private pendingFrame: Uint8ClampedArray | null = null;
  private animationFrameId: number | null = null;
  private lastRenderTime: number = 0;
  private frameCount: number = 0;
  private fpsCallback?: (fps: number) => void;

  setCanvas(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
    this.ctx = canvas.getContext("2d", { alpha: false });
    if (this.ctx) {
      this.ctx.imageSmoothingEnabled = false;
      this.imageData = this.ctx.createImageData(64, 64);
    }
  }

  setFpsCallback(callback: (fps: number) => void) {
    this.fpsCallback = callback;
  }

  renderFrame(
    data: number[] | Uint8Array,
    width: number = 64,
    height: number = 64,
  ) {
    if (!this.ctx || !this.imageData || !this.canvas) return;

    // Convertir en Uint8ClampedArray si nécessaire
    const frameData =
      data instanceof Uint8ClampedArray ? data : new Uint8ClampedArray(data);

    // Préparer les données RGBA
    const rgbaData = new Uint8ClampedArray(width * height * 4);
    for (let i = 0; i < width * height; i++) {
      rgbaData[i * 4] = frameData[i * 3]; // R
      rgbaData[i * 4 + 1] = frameData[i * 3 + 1]; // G
      rgbaData[i * 4 + 2] = frameData[i * 3 + 2]; // B
      rgbaData[i * 4 + 3] = 255; // A
    }

    this.pendingFrame = rgbaData;

    // Utiliser requestAnimationFrame pour un rendu fluide
    if (!this.animationFrameId) {
      this.animationFrameId = requestAnimationFrame(() => this.render());
    }
  }

  private render() {
    if (!this.pendingFrame || !this.ctx || !this.imageData) {
      this.animationFrameId = null;
      return;
    }

    // Mettre à jour l'ImageData
    this.imageData.data.set(this.pendingFrame);
    this.ctx.putImageData(this.imageData, 0, 0);

    // Calculer le FPS
    const now = performance.now();
    if (this.lastRenderTime > 0) {
      this.frameCount++;
      const elapsed = now - this.lastRenderTime;
      if (elapsed >= 1000) {
        const fps = Math.round((this.frameCount * 1000) / elapsed);
        this.fpsCallback?.(fps);
        this.frameCount = 0;
        this.lastRenderTime = now;
      }
    } else {
      this.lastRenderTime = now;
    }

    this.pendingFrame = null;
    this.animationFrameId = null;
  }

  destroy() {
    if (this.animationFrameId) {
      cancelAnimationFrame(this.animationFrameId);
    }
  }
}

export default function App() {
  const [ws, setWs] = useState<WebSocket | null>(null);
  const [connected, setConnected] = useState(false);
  const [currentEffect, setCurrentEffect] = useState<number>(0);
  const [currentColorMode, setCurrentColorMode] = useState<string>("rainbow");
  const [customColor, setCustomColor] = useState<string>("#ff0080");
  const [spectrum, setSpectrum] = useState<number[]>(new Array(64).fill(0));
  const [fps, setFps] = useState<number>(0);
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const frameRendererRef = useRef<FrameRenderer | null>(null);
  const decompressorRef = useRef<FrameDecompressor | null>(null);

  const effects: Effect[] = [
    { id: 0, name: "Plasma", icon: "🌊" },
    { id: 1, name: "Fire", icon: "🔥" },
    { id: 2, name: "Matrix", icon: "💊" },
    { id: 3, name: "Spectrum", icon: "📊" },
    { id: 4, name: "Starfield", icon: "⭐" },
    { id: 5, name: "Ripple", icon: "💧" },
    { id: 6, name: "Life", icon: "🧬" },
    { id: 7, name: "Mandelbrot", icon: "🌀" },
    { id: 8, name: "Lava", icon: "🌋" },
  ];

  const colorModes: ColorMode[] = [
    { id: "rainbow", name: "Rainbow", icon: "🌈" },
    { id: "ocean", name: "Ocean", icon: "🌊" },
    { id: "fire", name: "Fire", icon: "🔥" },
    { id: "matrix", name: "Matrix", icon: "💚" },
    { id: "custom", name: "Custom", icon: "🎨" },
  ];

  // Initialiser les composants optimisés
  useEffect(() => {
    frameRendererRef.current = new FrameRenderer();
    frameRendererRef.current.setFpsCallback(setFps);
    decompressorRef.current = new FrameDecompressor();

    return () => {
      frameRendererRef.current?.destroy();
    };
  }, []);

  // Configurer le canvas
  useEffect(() => {
    if (canvasRef.current && frameRendererRef.current) {
      frameRendererRef.current.setCanvas(canvasRef.current);
    }
  }, [canvasRef.current]);

  // Connexion WebSocket optimisée
  useEffect(() => {
    let reconnectTimeout: NodeJS.Timeout;
    let pingInterval: NodeJS.Timeout;
    let currentWs: WebSocket | null = null;

    const connect = () => {
      const websocket = new WebSocket("ws://localhost:8080");
      currentWs = websocket;
      websocket.binaryType = "arraybuffer";

      websocket.onopen = () => {
        console.log("Connected to visualizer");
        setConnected(true);

        // Ping pour maintenir la connexion
        pingInterval = setInterval(() => {
          if (websocket.readyState === WebSocket.OPEN) {
            websocket.send(JSON.stringify({ type: "ping" }));
          }
        }, 30000);
      };

      websocket.onmessage = async (event: MessageEvent) => {
        try {
          const data: WebSocketMessage = JSON.parse(event.data);

          switch (data.type) {
            case "frame":
              frameRendererRef.current?.renderFrame(data.data);
              break;

            case "compressedframe":
              if (decompressorRef.current) {
                const compressed = new Uint8Array(data.data);
                const decompressed =
                  await decompressorRef.current.decompress(compressed);
                frameRendererRef.current?.renderFrame(
                  decompressed,
                  data.width,
                  data.height,
                );
              }
              break;

            case "spectrum":
              // Limiter les mises à jour du spectre pour éviter trop de re-renders
              setSpectrum((prevSpectrum) => {
                // Mise à jour seulement si significativement différent
                const isDifferent = data.data.some(
                  (v, i) => Math.abs(v - prevSpectrum[i]) > 0.05,
                );
                return isDifferent ? data.data : prevSpectrum;
              });
              break;
          }
        } catch (error) {
          console.error("Error processing message:", error);
        }
      };

      websocket.onerror = (error) => {
        console.error("WebSocket error:", error);
      };

      websocket.onclose = () => {
        setConnected(false);
        clearInterval(pingInterval);

        // Reconnexion automatique après 2 secondes
        reconnectTimeout = setTimeout(connect, 2000);
      };

      setWs(websocket);
    };

    connect();

    return () => {
      clearTimeout(reconnectTimeout);
      clearInterval(pingInterval);
      currentWs?.close();
    };
  }, []);

  // Optimisation des callbacks avec useCallback
  const selectEffect = useCallback(
    (id: number) => {
      setCurrentEffect(id);
      if (ws && ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({ type: "effect", id }));
      }
    },
    [ws],
  );

  const selectColorMode = useCallback(
    (mode: string) => {
      setCurrentColorMode(mode);
      if (ws && ws.readyState === WebSocket.OPEN) {
        ws.send(
          JSON.stringify({
            type: "param",
            name: "colorMode",
            value: mode,
          }),
        );
      }
    },
    [ws],
  );

  const updateCustomColor = useCallback(
    (r: number, g: number, b: number) => {
      if (
        ws &&
        ws.readyState === WebSocket.OPEN &&
        currentColorMode === "custom"
      ) {
        const rNorm = r / 255;
        const gNorm = g / 255;
        const bNorm = b / 255;
        ws.send(
          JSON.stringify({
            type: "param",
            name: "customColor",
            value: `${rNorm},${gNorm},${bNorm}`,
          }),
        );
      }
    },
    [ws, currentColorMode],
  );

  return (
    <div className="container">
      {/* Header avec statut de connexion */}
      <div className="header">
        <h1>
          <span className="logo">🌈</span> LED Visualizer
        </h1>
        <div className="status">
          <span className={connected ? "connected" : "disconnected"}>
            {connected ? "● Connected" : "○ Disconnected"}
          </span>
          {connected && fps > 0 && <span className="fps">{fps} FPS</span>}
        </div>
      </div>

      {/* Prévisualisation LED */}
      <div className="preview-section">
        <div className="preview-container">
          <canvas
            ref={canvasRef}
            width={64}
            height={64}
            className="led-preview"
          />
          <div className="preview-overlay">
            <div className="preview-info">128×128 LED Matrix</div>
          </div>
        </div>
      </div>

      {/* Analyseur de spectre optimisé */}
      <div className="spectrum-section">
        <h2>Audio Spectrum</h2>
        <div className="spectrum-bars">
          {spectrum.map((value, index) => (
            <div
              key={index}
              className="spectrum-bar"
              style={{
                height: `${Math.max(2, value * 100)}%`,
                backgroundColor: `hsl(${(index / spectrum.length) * 360}, 70%, 50%)`,
              }}
            />
          ))}
        </div>
      </div>

      {/* Sélection des effets */}
      <div className="effects-section">
        <h2>Effects</h2>
        <div className="effects-grid">
          {effects.map((effect) => (
            <button
              key={effect.id}
              className={`effect-button ${
                currentEffect === effect.id ? "active" : ""
              }`}
              onClick={() => selectEffect(effect.id)}
              disabled={!connected}
            >
              <span className="effect-icon">{effect.icon}</span>
              <span className="effect-name">{effect.name}</span>
            </button>
          ))}
        </div>
      </div>

      {/* Modes de couleur */}
      <div className="color-section">
        <h2>Color Mode</h2>
        <div className="color-modes">
          {colorModes.map((mode) => (
            <button
              key={mode.id}
              className={`color-mode-button ${
                currentColorMode === mode.id ? "active" : ""
              }`}
              onClick={() => selectColorMode(mode.id)}
              disabled={!connected}
            >
              <span className="mode-icon">{mode.icon}</span>
              <span className="mode-name">{mode.name}</span>
            </button>
          ))}
        </div>

        {/* Sélecteur de couleur personnalisée */}
        {currentColorMode === "custom" && (
          <div className="custom-color-picker">
            <input
              type="color"
              value={customColor}
              onChange={(e) => {
                setCustomColor(e.target.value);
                const r = parseInt(e.target.value.slice(1, 3), 16);
                const g = parseInt(e.target.value.slice(3, 5), 16);
                const b = parseInt(e.target.value.slice(5, 7), 16);
                updateCustomColor(r, g, b);
              }}
              disabled={!connected}
            />
            <span>Custom Color</span>
          </div>
        )}
      </div>
    </div>
  );
}
