// Two-color theming: "main" drives the background and every surface shade derived from it;
// "accent" colors buttons, sliders and highlights. Text flips light/dark automatically.

export type Theme = { main: string; accent: string };

export const DEFAULT_THEME: Theme = { main: "#000000", accent: "#8b7bff" };

export const PRESETS: (Theme & { name: string })[] = [
  { name: "Midnight", main: "#000000", accent: "#8b7bff" },
  { name: "Forest", main: "#0a1711", accent: "#3ddc84" },
  { name: "Ocean", main: "#06131d", accent: "#38bdf8" },
  { name: "Ember", main: "#170c07", accent: "#ff7a45" },
  { name: "Rose", main: "#170a10", accent: "#ff5c9a" },
  { name: "Paper", main: "#f3f0e8", accent: "#3b5bdb" },
];

const KEY = "theme";

function rgb(hex: string): [number, number, number] {
  const n = parseInt(hex.replace("#", "").padEnd(6, "0").slice(0, 6), 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}

function hex([r, g, b]: number[]): string {
  return "#" + [r, g, b].map((v) => Math.round(v).toString(16).padStart(2, "0")).join("");
}

/** Blend `a` toward `b` by `t` (0..1). */
function mix(a: string, b: string, t: number): string {
  const x = rgb(a);
  const y = rgb(b);
  return hex(x.map((v, i) => v + (y[i] - v) * t));
}

/** Relative luminance (WCAG), 0 = black, 1 = white. */
function luminance(c: string): number {
  const [r, g, b] = rgb(c).map((v) => {
    const s = v / 255;
    return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

const isHex = (s: unknown): s is string => typeof s === "string" && /^#[0-9a-f]{6}$/i.test(s);

export function loadTheme(): Theme {
  try {
    const t = JSON.parse(localStorage.getItem(KEY) ?? "null");
    if (isHex(t?.main) && isHex(t?.accent)) return { main: t.main, accent: t.accent };
  } catch {}
  return { ...DEFAULT_THEME };
}

export function saveTheme(t: Theme) {
  try {
    localStorage.setItem(KEY, JSON.stringify(t));
  } catch {}
}

export function applyTheme({ main, accent }: Theme) {
  const light = luminance(main) > 0.4;
  const text = light ? "#141414" : "#ffffff";
  const vars: Record<string, string> = {
    "--bg": main,
    "--surface": mix(main, text, 0.06),
    "--surface-2": mix(main, text, 0.13),
    "--surface-3": mix(main, text, 0.3),
    "--hover": mix(main, text, 0.09),
    "--text": text,
    "--muted": mix(text, main, 0.38),
    "--accent": accent,
    "--on-accent": luminance(accent) > 0.35 ? "#000000" : "#ffffff",
  };
  const root = document.documentElement;
  for (const [k, v] of Object.entries(vars)) root.style.setProperty(k, v);
  root.style.colorScheme = light ? "light" : "dark";
  // Lets the title bar recolor the taskbar icon.
  window.dispatchEvent(new CustomEvent("xstify-theme", { detail: accent }));
}
