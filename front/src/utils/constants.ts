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
    { id: 1, name: 'Pulse', emoji: '💓' },
    { id: 2, name: 'Wave', emoji: '🌊' },
    { id: 3, name: 'Strobe', emoji: '⚡' },
    { id: 4, name: 'Rainbow', emoji: '🌈' },
    { id: 5, name: 'Matrix', emoji: '🔢' },
    { id: 6, name: 'Fire', emoji: '🔥' },
    { id: 7, name: 'Ocean', emoji: '🌊' },
    { id: 8, name: 'Space', emoji: '🌌' },
];

export const COLOR_MODES: ColorMode[] = [
    { value: 'rainbow', label: 'Rainbow', emoji: '🌈' },
    { value: 'solid', label: 'Solid', emoji: '🔵' },
    { value: 'pulse', label: 'Pulse', emoji: '💓' },
    { value: 'strobe', label: 'Strobe', emoji: '⚡' },
    { value: 'fade', label: 'Fade', emoji: '🌅' },
];

export const COLOR_CHANNELS: ColorChannel[] = [
    { key: 'r', name: 'Red', emoji: '🔴' },
    { key: 'g', name: 'Green', emoji: '🟢' },
    { key: 'b', name: 'Blue', emoji: '🔵' },
];
