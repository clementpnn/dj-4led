import { invoke } from "@tauri-apps/api/core";
import { ref } from "vue";
import { EffectResult } from "../types";

export function useEffects() {
  const currentEffect = ref<number | undefined>(undefined);
  const loading = ref<boolean>(false);

  const setEffect = async (effectId: number): Promise<EffectResult> => {
    loading.value = true;
    try {
      const result = await invoke<string>("dj_set_effect", { effectId });
      currentEffect.value = effectId;
      return { success: true, message: result };
    } catch (error) {
      return { success: false, message: `❌ Effect error: ${error}` };
    } finally {
      loading.value = false;
    }
  };

  const resetEffect = (): void => {
    currentEffect.value = undefined;
  };

  return {
    currentEffect,
    loading,
    setEffect,
    resetEffect,
  };
}
