// XSTIFY's logo: an "X" inside a ring, drawn in one color so it can follow the theme accent.
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Image } from "@tauri-apps/api/image";

export const LOGO_VIEWBOX = "0 0 32 32";
export const LOGO_INNER = `<circle cx="16" cy="16" r="13.5" fill="none" stroke="currentColor" stroke-width="3"/><path d="M11 11l10 10M21 11L11 21" fill="none" stroke="currentColor" stroke-width="3.4" stroke-linecap="round"/>`;

/** Recolor the window/taskbar icon to `color` (renders the logo to pixels). */
export async function setTaskbarIcon(color: string) {
  try {
    const size = 64;
    const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="${LOGO_VIEWBOX}" width="${size}" height="${size}" style="color:${color}">${LOGO_INNER.replaceAll("currentColor", color)}</svg>`;
    const img = new window.Image();
    img.src = "data:image/svg+xml;charset=utf-8," + encodeURIComponent(svg);
    await img.decode();
    const canvas = document.createElement("canvas");
    canvas.width = canvas.height = size;
    const ctx = canvas.getContext("2d")!;
    ctx.drawImage(img, 0, 0, size, size);
    const rgba = ctx.getImageData(0, 0, size, size).data;
    await getCurrentWindow().setIcon(await Image.new(new Uint8Array(rgba.buffer), size, size));
  } catch {
    // Not critical: keep the default icon.
  }
}
