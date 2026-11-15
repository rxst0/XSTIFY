// Which Spotify account the user has: chosen on first launch, checked against Spotify after
// sign-in, changeable in Settings.
//  - premium: songs play inside XSTIFY (Spotify's Web Playback SDK, Premium-only)
//  - free:    songs open in Spotify's official web player window (free tier, with Spotify's ads)
import { invoke } from "@tauri-apps/api/core";

export type Plan = "premium" | "free";
const KEY = "plan";

function load(): Plan | null {
  try {
    const v = localStorage.getItem(KEY);
    return v === "premium" || v === "free" ? v : null;
  } catch {
    return null;
  }
}

class PlanState {
  value = $state<Plan | null>(load());
  /** Account type reported by Spotify after sign-in ("premium", "free", …). */
  product = $state<string | null>(null);

  get isFree() {
    return this.value === "free";
  }

  set(p: Plan) {
    this.value = p;
    try {
      localStorage.setItem(KEY, p);
    } catch {}
  }

  /** Open a Spotify song/album/playlist/artist in Spotify's web player (Free mode). */
  openWeb(uri: string) {
    const [, type, id] = uri.split(":");
    const url = type && id ? `https://open.spotify.com/${type}/${id}` : "https://open.spotify.com/";
    return invoke("open_web_player", { url });
  }
}

export const plan = new PlanState();
