// src/composables/useEffects.ts
import { invoke } from "@tauri-apps/api/core";
import { ref } from "vue";
import { useConfig } from "./useConfig";

interface EffectResult {
  success: boolean;
  message: string;
}

export function useEffects() {
  const currentEffect = ref<number | null>(null);
  const loading = ref<boolean>(false);
  const { config } = useConfig();

  const setEffect = async (effectId: number): Promise<EffectResult> => {
    loading.value = true;
    try {
      const result = await invoke<string>("dj_set_effect", {
        effectId,
        serverIp: config.value.server.ip,
        serverPort: config.value.server.port,
      });
      currentEffect.value = effectId;
      return { success: true, message: result };
    } catch (error) {
      return { success: false, message: `❌ Effect error: ${error}` };
    } finally {
      loading.value = false;
    }
  };

  const resetEffect = (): void => {
    currentEffect.value = null;
  };

  return {
    currentEffect,
    loading,
    setEffect,
    resetEffect,
  };
}
