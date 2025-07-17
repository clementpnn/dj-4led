<template>
  <div class="settings-panel">
    <h2>⚙️ Configuration</h2>

    <!-- Server Configuration -->
    <div class="config-section">
      <h3>🌐 Serveur UDP</h3>
      <div class="input-group">
        <label for="server-ip">Adresse IP:</label>
        <input
          id="server-ip"
          v-model="serverConfig.ip"
          type="text"
          placeholder="127.0.0.1"
          class="input-field"
        />
      </div>
      <div class="input-group">
        <label for="server-port">Port:</label>
        <input
          id="server-port"
          v-model.number="serverConfig.port"
          type="number"
          placeholder="8081"
          class="input-field"
        />
      </div>
    </div>

    <!-- Audio Configuration -->
    <div class="config-section">
      <h3>🎤 Entrée Audio</h3>
      <div class="input-group">
        <label for="audio-device">Dispositif:</label>
        <select
          id="audio-device"
          v-model="audioConfig.deviceId"
          class="input-field"
        >
          <option value="">Par défaut</option>
          <option
            v-for="device in audioDevices"
            :key="device.id"
            :value="device.id"
          >
            {{ device.name }}
          </option>
        </select>
      </div>
      <div class="input-group">
        <label for="audio-gain">Gain:</label>
        <input
          id="audio-gain"
          v-model.number="audioConfig.gain"
          type="range"
          min="0.1"
          max="5"
          step="0.1"
          class="slider"
        />
        <span class="value">{{ audioConfig.gain.toFixed(1) }}</span>
      </div>
    </div>

    <!-- LED Controllers Configuration -->
    <div class="config-section">
      <h3>💡 Contrôleurs LED</h3>
      <div class="controllers-info">
        <p>Nombre de contrôleurs: {{ ledControllers.length }}</p>
        <p>Total de LEDs: {{ totalLeds }}</p>
      </div>

      <div
        v-for="(controller, index) in ledControllers"
        :key="index"
        class="controller-config"
      >
        <div class="controller-header">
          <h4>Contrôleur {{ index + 1 }}</h4>
          <button
            @click="removeController(index)"
            class="btn-remove"
            :disabled="ledControllers.length <= 1"
          >
            ❌
          </button>
        </div>

        <div class="input-group">
          <label>IP:</label>
          <input
            v-model="controller.ip"
            type="text"
            placeholder="192.168.1.45"
            class="input-field"
          />
        </div>
        <div class="input-group">
          <label>Port:</label>
          <input
            v-model.number="controller.port"
            type="number"
            placeholder="6454"
            class="input-field small"
          />
        </div>
      </div>

      <button @click="addController" class="btn-add">
        ➕ Ajouter un contrôleur
      </button>
    </div>

    <!-- Display Configuration -->
    <div class="config-section">
      <h3>🖼️ Affichage</h3>
      <div class="input-group">
        <label for="display-mode">Mode d'affichage:</label>
        <select
          id="display-mode"
          v-model="displayConfig.mode"
          class="input-field"
        >
          <option value="auto">Auto (adaptatif)</option>
          <option value="grid">Grille</option>
          <option value="strip">Bande</option>
          <option value="circular">Circulaire</option>
        </select>
      </div>
      <div class="input-group">
        <label for="display-scale">Échelle:</label>
        <input
          id="display-scale"
          v-model.number="displayConfig.scale"
          type="range"
          min="0.5"
          max="2"
          step="0.1"
          class="slider"
        />
        <span class="value">{{ displayConfig.scale.toFixed(1) }}x</span>
      </div>
    </div>

    <!-- Action Buttons -->
    <div class="actions">
      <button @click="saveConfig" class="btn-primary">💾 Sauvegarder</button>
      <button @click="resetConfig" class="btn-secondary">
        🔄 Réinitialiser
      </button>
      <button @click="testConnection" class="btn-secondary">
        🧪 Tester la connexion
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface AudioDevice {
  id: string;
  name: string;
}

interface LedController {
  ip: string;
  port: number;
}

interface ServerConfig {
  ip: string;
  port: number;
}

interface AudioConfig {
  deviceId: string;
  gain: number;
}

interface DisplayConfig {
  mode: "auto" | "grid" | "strip" | "circular";
  scale: number;
}

const emit = defineEmits<{
  "config-updated": [
    config: {
      server: ServerConfig;
      audio: AudioConfig;
      controllers: LedController[];
      display: DisplayConfig;
    },
  ];
  log: [message: string, type: "info" | "error" | "success"];
}>();

// State
const serverConfig = ref<ServerConfig>({
  ip: "127.0.0.1",
  port: 8081,
});

const audioConfig = ref<AudioConfig>({
  deviceId: "",
  gain: 2.0,
});

const audioDevices = ref<AudioDevice[]>([]);

const ledControllers = ref<LedController[]>([
  {
    ip: "192.168.1.45",
    port: 6454,
  },
]);

const displayConfig = ref<DisplayConfig>({
  mode: "auto",
  scale: 1.0,
});

// Computed
const totalLeds = computed(() => {
  // Les LEDs sont gérées automatiquement par le backend
  return ledControllers.value.length * 4096; // Estimation: 32x128 par contrôleur
});

// Methods
const addController = () => {
  const lastController =
    ledControllers.value.length > 0
      ? ledControllers.value[ledControllers.value.length - 1]
      : {
          ip: "192.168.1.45",
          port: 6454,
        };
  ledControllers.value.push({
    ip: lastController.ip,
    port: lastController.port,
  });
};

const removeController = (index: number) => {
  if (ledControllers.value.length > 1) {
    ledControllers.value.splice(index, 1);
  }
};

const saveConfig = async () => {
  try {
    const config = {
      server: serverConfig.value,
      audio: audioConfig.value,
      controllers: ledControllers.value,
      display: displayConfig.value,
    };

    // Sauvegarder dans le localStorage pour persistance
    localStorage.setItem("dj4led-config", JSON.stringify(config));

    emit("config-updated", config);
    emit("log", "✅ Configuration sauvegardée", "success");
  } catch (error) {
    emit("log", `❌ Erreur de sauvegarde: ${error}`, "error");
  }
};

const resetConfig = () => {
  serverConfig.value = { ip: "127.0.0.1", port: 8081 };
  audioConfig.value = { deviceId: "", gain: 2.0 };
  ledControllers.value = [
    {
      ip: "192.168.1.45",
      port: 6454,
    },
  ];
  displayConfig.value = { mode: "auto", scale: 1.0 };
  emit("log", "🔄 Configuration réinitialisée", "info");
};

const testConnection = async () => {
  try {
    const result = await invoke("dj_test_connection", {
      serverIp: serverConfig.value.ip,
      serverPort: serverConfig.value.port,
    });
    emit("log", `🧪 Test de connexion: ${result}`, "info");
  } catch (error) {
    emit("log", `❌ Erreur de test: ${error}`, "error");
  }
};

const loadAudioDevices = async () => {
  try {
    const devices = await invoke<AudioDevice[]>("dj_list_audio_devices");
    audioDevices.value = devices;
  } catch (error) {
    console.error("Failed to load audio devices:", error);
    // Fallback devices for demo
    audioDevices.value = [
      { id: "default", name: "Microphone par défaut" },
      { id: "builtin", name: "Microphone intégré" },
      { id: "usb", name: "Microphone USB" },
    ];
  }
};

const loadConfig = () => {
  const saved = localStorage.getItem("dj4led-config");
  if (saved) {
    try {
      const config = JSON.parse(saved);
      if (config.server) serverConfig.value = config.server;
      if (config.audio) audioConfig.value = config.audio;
      if (config.controllers) ledControllers.value = config.controllers;
      if (config.display) displayConfig.value = config.display;
    } catch (error) {
      console.error("Failed to load config:", error);
    }
  }
};

// Lifecycle
onMounted(() => {
  loadConfig();
  loadAudioDevices();
});
</script>

<style scoped>
.settings-panel {
  background: rgba(0, 0, 0, 0.8);
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 10px;
  padding: 20px;
  max-width: 600px;
  max-height: 80vh;
  overflow-y: auto;
}

.settings-panel h2 {
  color: #fff;
  margin: 0 0 20px 0;
  font-size: 24px;
  text-align: center;
}

.config-section {
  background: rgba(255, 255, 255, 0.05);
  border-radius: 8px;
  padding: 15px;
  margin-bottom: 20px;
}

.config-section h3 {
  color: #fff;
  margin: 0 0 15px 0;
  font-size: 18px;
}

.input-group {
  display: flex;
  align-items: center;
  margin-bottom: 10px;
  gap: 10px;
}

.input-group label {
  color: rgba(255, 255, 255, 0.8);
  min-width: 120px;
  font-size: 14px;
}

.input-field {
  flex: 1;
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 4px;
  padding: 8px 12px;
  color: #fff;
  font-size: 14px;
  transition: all 0.3s ease;
}

.input-field:focus {
  outline: none;
  border-color: #00ff88;
  background: rgba(255, 255, 255, 0.15);
}

.input-field.small {
  flex: 0 0 80px;
}

.slider {
  flex: 1;
  -webkit-appearance: none;
  appearance: none;
  height: 4px;
  background: rgba(255, 255, 255, 0.2);
  border-radius: 2px;
  outline: none;
}

.slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 16px;
  height: 16px;
  background: #00ff88;
  border-radius: 50%;
  cursor: pointer;
}

.slider::-moz-range-thumb {
  width: 16px;
  height: 16px;
  background: #00ff88;
  border-radius: 50%;
  cursor: pointer;
}

.value {
  color: #00ff88;
  font-weight: bold;
  min-width: 50px;
  text-align: right;
}

.controllers-info {
  background: rgba(0, 255, 136, 0.1);
  border: 1px solid rgba(0, 255, 136, 0.3);
  border-radius: 4px;
  padding: 10px;
  margin-bottom: 15px;
}

.controllers-info p {
  color: #00ff88;
  margin: 5px 0;
  font-size: 14px;
}

.controller-config {
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  padding: 12px;
  margin-bottom: 10px;
}

.controller-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.controller-header h4 {
  color: rgba(255, 255, 255, 0.9);
  margin: 0;
  font-size: 16px;
}

.btn-remove {
  background: none;
  border: none;
  cursor: pointer;
  font-size: 14px;
  padding: 4px;
  transition: transform 0.2s;
}

.btn-remove:hover:not(:disabled) {
  transform: scale(1.2);
}

.btn-remove:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.btn-add {
  width: 100%;
  background: rgba(0, 255, 136, 0.2);
  border: 1px solid rgba(0, 255, 136, 0.5);
  color: #00ff88;
  padding: 10px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.3s ease;
}

.btn-add:hover {
  background: rgba(0, 255, 136, 0.3);
  transform: translateY(-1px);
}

.actions {
  display: flex;
  gap: 10px;
  margin-top: 20px;
}

.btn-primary,
.btn-secondary {
  flex: 1;
  padding: 12px;
  border: none;
  border-radius: 6px;
  font-size: 16px;
  font-weight: bold;
  cursor: pointer;
  transition: all 0.3s ease;
}

.btn-primary {
  background: linear-gradient(45deg, #00ff88, #00cc66);
  color: #000;
}

.btn-primary:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 20px rgba(0, 255, 136, 0.5);
}

.btn-secondary {
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.2);
  color: #fff;
}

.btn-secondary:hover {
  background: rgba(255, 255, 255, 0.2);
  transform: translateY(-1px);
}

/* Scrollbar styling */
.settings-panel::-webkit-scrollbar {
  width: 8px;
}

.settings-panel::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.05);
  border-radius: 4px;
}

.settings-panel::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.2);
  border-radius: 4px;
}

.settings-panel::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.3);
}
</style>
