import { useEffect, useRef, useState } from "react";

type Effect = {
  id: number;
  name: string;
  icon: string;
};

type FrameMessage = {
  type: "frame";
  data: number[];
};

type SpectrumMessage = {
  type: "spectrum";
  data: number[];
};

type WebSocketMessage = FrameMessage | SpectrumMessage;

export default function App() {
  const [ws, setWs] = useState<WebSocket | null>(null);
  const [connected, setConnected] = useState(false);
  const [currentEffect, setCurrentEffect] = useState<number>(0);
  const [spectrum, setSpectrum] = useState<number[]>(new Array(64).fill(0));
  const [frameData, setFrameData] = useState<number[] | null>(null);
  const canvasRef = useRef<HTMLCanvasElement | null>(null);

  const effects: Effect[] = [
    { id: 0, name: "Spectrum Bars", icon: "📊" },
    { id: 1, name: "Circular Wave", icon: "🌊" },
    { id: 2, name: "Particles", icon: "✨" },
  ];

  useEffect(() => {
    const websocket = new WebSocket("ws://localhost:8080");

    websocket.onopen = () => {
      console.log("Connected to visualizer");
      setConnected(true);
    };

    websocket.onmessage = (event: MessageEvent) => {
      const data: WebSocketMessage = JSON.parse(event.data);

      switch (data.type) {
        case "frame":
          setFrameData(data.data);
          break;
        case "spectrum":
          setSpectrum(data.data);
          break;
      }
    };

    websocket.onclose = () => {
      setConnected(false);
    };

    setWs(websocket);

    return () => websocket.close();
  }, []);

  const drawFrame = () => {
    const canvas = canvasRef.current;
    if (!canvas || !frameData) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const imageData = ctx.createImageData(64, 64);

    for (let i = 0; i < frameData.length; i += 3) {
      const pixelIndex = (i / 3) * 4;
      imageData.data[pixelIndex] = frameData[i];
      imageData.data[pixelIndex + 1] = frameData[i + 1];
      imageData.data[pixelIndex + 2] = frameData[i + 2];
      imageData.data[pixelIndex + 3] = 255;
    }

    ctx.putImageData(imageData, 0, 0);
  };

  // Auto-refresh canvas when frame data changes
  useEffect(() => {
    drawFrame();
  }, [frameData]);

  const selectEffect = (id: number) => {
    setCurrentEffect(id);
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify({ type: "Effect", id }));
    }
  };

  return (
    <div
      style={{
        minHeight: "100vh",
        background: "linear-gradient(135deg, #667eea 0%, #764ba2 100%)",
        color: "white",
        fontFamily: "Arial, sans-serif",
      }}
    >
      <header
        style={{
          padding: "20px",
          textAlign: "center",
          borderBottom: "1px solid rgba(255,255,255,0.2)",
        }}
      >
        <h1 style={{ margin: 0, fontSize: "2.5rem" }}>🎵 LED Visualizer</h1>
        <p
          style={{
            margin: "10px 0 0 0",
            opacity: 0.8,
          }}
        >
          Status: {connected ? "🟢 Connected" : "🔴 Disconnected"}
        </p>
      </header>

      <main
        style={{
          display: "flex",
          gap: "20px",
          padding: "20px",
          maxWidth: "1200px",
          margin: "0 auto",
        }}
      >
        {/* LED Preview */}
        <div
          style={{
            flex: 1,
            background: "rgba(255,255,255,0.1)",
            borderRadius: "10px",
            padding: "20px",
          }}
        >
          <h2 style={{ marginTop: 0 }}>💡 LED Preview</h2>
          <canvas
            ref={canvasRef}
            width={64}
            height={64}
            style={{
              width: "100%",
              maxWidth: "400px",
              height: "auto",
              border: "2px solid rgba(255,255,255,0.3)",
              borderRadius: "8px",
              imageRendering: "pixelated",
            }}
          />
        </div>

        {/* Controls */}
        <div
          style={{
            flex: 1,
            background: "rgba(255,255,255,0.1)",
            borderRadius: "10px",
            padding: "20px",
          }}
        >
          <h2 style={{ marginTop: 0 }}>🎛️ Effects</h2>
          <div
            style={{
              display: "grid",
              gap: "10px",
              marginBottom: "30px",
            }}
          >
            {effects.map((effect) => (
              <button
                key={effect.id}
                onClick={() => selectEffect(effect.id)}
                style={{
                  padding: "15px",
                  border: "none",
                  borderRadius: "8px",
                  background:
                    currentEffect === effect.id
                      ? "rgba(255,255,255,0.3)"
                      : "rgba(255,255,255,0.1)",
                  color: "white",
                  fontSize: "16px",
                  cursor: "pointer",
                  transition: "all 0.2s",
                  textAlign: "left",
                }}
              >
                <span style={{ fontSize: "24px", marginRight: "10px" }}>
                  {effect.icon}
                </span>
                {effect.name}
              </button>
            ))}
          </div>

          <h3>📊 Audio Spectrum</h3>
          <div
            style={{
              display: "flex",
              alignItems: "end",
              height: "100px",
              gap: "2px",
              background: "rgba(0,0,0,0.3)",
              padding: "10px",
              borderRadius: "8px",
            }}
          >
            {spectrum.map((value, index) => (
              <div
                key={index}
                style={{
                  flex: 1,
                  background: `hsl(${(index * 360) / spectrum.length}, 70%, 50%)`,
                  height: `${value * 100}%`,
                  minHeight: "2px",
                  borderRadius: "2px 2px 0 0",
                }}
              />
            ))}
          </div>
        </div>
      </main>
    </div>
  );
}
