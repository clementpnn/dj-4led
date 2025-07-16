// src/utils/constants.ts

export interface Effect {
    id: number;
    name: string;
    emoji: string;
}

export interface ColorMode {
    value: string;
    label: string;
    emoji: string;
}

export interface ColorChannel {
    key: 'r' | 'g' | 'b';
    name: string;
    emoji: string;
}

export const EFFECTS: Effect[] = [
    { id: 0, name: 'Wave', emoji: '🌊' },
    { id: 1, name: 'Pulse', emoji: '💓' },
    { id: 2, name: 'Strobe', emoji: '⚡' },
    { id: 3, name: 'Rainbow', emoji: '🌈' },
    { id: 4, name: 'Matrix', emoji: '🔢' },
    { id: 5, name: 'Fire', emoji: '🔥' },
    { id: 6, name: 'Ocean', emoji: '🌊' },
    { id: 7, name: 'Space', emoji: '🌌' },
];

export const COLOR_MODES: ColorMode[] = [
    { value: 'rainbow', label: 'Rainbow', emoji: '🌈' },
    { value: 'fire', label: 'Fire', emoji: '🔥' },
    { value: 'ocean', label: 'Ocean', emoji: '🌊' },
    { value: 'sunset', label: 'Sunset', emoji: '🌅' },
    { value: 'matrix', label: 'Matrix', emoji: '🌿' },
    { value: 'custom', label: 'Custom', emoji: '🎨' },
];

export const COLOR_CHANNELS: ColorChannel[] = [
    { key: 'r', name: 'Red', emoji: '🔴' },
    { key: 'g', name: 'Green', emoji: '🟢' },
    { key: 'b', name: 'Blue', emoji: '🔵' },
];
