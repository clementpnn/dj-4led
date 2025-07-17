// src/composables/useColors.ts
import { invoke } from "@tauri-apps/api/core";
import { computed, ref } from "vue";

interface CustomColor {
  r: number;
  g: number;
  b: number;
}

interface ColorResult {
  success: boolean;
  message: string;
}

export function useColors() {
  const currentMode = ref<string | undefined>(undefined);
  const customColor = ref<CustomColor>({ r: 1, g: 0.5, b: 0 });
  const loading = ref<boolean>(false);

  const colorPreviewStyle = computed(() => {
    const r = Math.round(customColor.value.r * 255);
    const g = Math.round(customColor.value.g * 255);
    const b = Math.round(customColor.value.b * 255);
    return {
      backgroundColor: `rgb(${r}, ${g}, ${b})`,
    };
  });

  const setColorMode = async (mode: string): Promise<ColorResult> => {
    loading.value = true;
    try {
      const result = await invoke<string>("dj_set_color_mode", { mode });
      currentMode.value = mode;
      return { success: true, message: result };
    } catch (error) {
      return { success: false, message: `❌ Mode error: ${error}` };
    } finally {
      loading.value = false;
    }
  };

  const setCustomColor = async (): Promise<ColorResult> => {
    loading.value = true;
    try {
      const { r, g, b } = customColor.value;
      const result = await invoke<string>("dj_set_custom_color", { r, g, b });
      return { success: true, message: result };
    } catch (error) {
      return { success: false, message: `❌ Color error: ${error}` };
    } finally {
      loading.value = false;
    }
  };

  const resetColors = (): void => {
    currentMode.value = undefined;
    customColor.value = { r: 1, g: 0.5, b: 0 };
  };

  return {
    currentMode,
    customColor,
    loading,
    colorPreviewStyle,
    setColorMode,
    setCustomColor,
    resetColors,
  };
}
