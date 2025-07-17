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
  key: "r" | "g" | "b";
  name: string;
  emoji: string;
}

export interface CustomColor {
  r: number;
  g: number;
  b: number;
}

// Ces constantes sont importées depuis le frontend pour référence
// mais peuvent être redéfinies ici si nécessaire
export const COLOR_CHANNELS: ColorChannel[] = [
  { key: "r", name: "Red", emoji: "🔴" },
  { key: "g", name: "Green", emoji: "🟢" },
  { key: "b", name: "Blue", emoji: "🔵" },
];
