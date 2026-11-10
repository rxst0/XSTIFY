// Built-in 10-band equalizer (engine in src-tauri/src/eq.rs).
// When on, Spotify's player frame sends its unprocessed audio to a silent output and
// XSTIFY plays the equalized copy on your speakers. This file keeps the sliders, presets
// and on/off switch, and tells the player frame which output to use.
import { invoke } from "@tauri-apps/api/core";
import { errMsg } from "./api";
import { player, toast } from "./state.svelte";

export const BANDS = ["31", "62", "125", "250", "500", "1k", "2k", "4k", "8k", "16k"];

export const PRESETS: Record<string, number[]> = {
  Flat: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
  "Bass Boost": [6, 5, 4, 2, 0, 0, 0, 0, 0, 0],
  Treble: [0, 0, 0, 0, 0, 0, 2, 4, 5, 6],
  Vocal: [-2, -2, -1, 1, 3, 4, 3, 1, 0, -1],
  Rock: [4, 3, 1, -1, -2, -1, 1, 3, 4, 4],
  Electronic: [5, 4, 1, 0, -2, 1, 0, 1, 4, 5],
  "Late Night": [-2, -1, 0, 1, 2, 2, 1, 0, -1, -2],
};

const KEY = "eq";
const ON_KEY = "eqOn";
const PLAYER_ORIGIN = "https://sdk.scdn.co";

function load(): number[] {
  try {
    const g = JSON.parse(localStorage.getItem(KEY) ?? "null");
    if (Array.isArray(g) && g.length === 10 && g.every((v) => typeof v === "number")) return g;
  } catch {}
  return [...PRESETS.Flat];
}

function store(key: string, value: string) {
  try {
    localStorage.setItem(key, value);
  } catch {}
}

/** Ask Spotify's player frame to send its audio to `label` ("" = normal speakers). */
function routePlayer(label: string): Promise<void> {
  const frame = document.querySelector<HTMLIFrameElement>(`iframe[src^="${PLAYER_ORIGIN}"]`);
  if (!frame?.contentWindow) return Promise.reject("Play a song first, then turn on the equalizer.");
  const id = Math.random().toString(36).slice(2);
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => done(new Error("The player didn't respond.")), 8000);
    function done(err?: Error) {
      clearTimeout(timer);
      window.removeEventListener("message", onMsg);
      err ? reject(err) : resolve();
    }
    function onMsg(e: MessageEvent) {
      if (e.origin !== PLAYER_ORIGIN || e.data?.xstify !== "sink-reply" || e.data.id !== id) return;
      done(e.data.ok ? undefined : new Error(e.data.error));
    }
    window.addEventListener("message", onMsg);
    frame.contentWindow!.postMessage({ xstify: "sink", id, label }, PLAYER_ORIGIN);
  });
}

class Equalizer {
  available = $state(true); // assume yes until checked
  output = $state<string | null>(null);
  enabled = $state(false);
  busy = $state(false);
  gains = $state<number[]>(load());
  #timer: ReturnType<typeof setTimeout> | undefined;

  /** Name of the preset matching the current sliders, or "Custom". */
  preset = $derived(
    Object.entries(PRESETS).find(([, g]) => g.every((v, i) => v === this.gains[i]))?.[0] ?? "Custom",
  );

  async refresh() {
    try {
      const s = await invoke<{ available: boolean; output: string | null }>("eq_status");
      this.available = s.available;
      this.output = s.output;
    } catch {}
  }

  /** Turn the EQ back on at startup if it was on last time. */
  restore() {
    try {
      if (localStorage.getItem(ON_KEY) === "1") this.setEnabled(true, true);
    } catch {}
  }

  async setEnabled(on: boolean, quiet = false) {
    if (this.busy || on === this.enabled) return;
    this.busy = true;
    try {
      if (on) {
        const output = await invoke<string>("eq_start", { gains: this.gains });
        try {
          await routePlayer(output);
          await player.setLocalOutput(output); // Local Files too
        } catch (e) {
          await routePlayer("").catch(() => {});
          await player.setLocalOutput("").catch(() => {});
          await invoke("eq_stop");
          throw e;
        }
        this.output = output;
        this.enabled = true;
      } else {
        await routePlayer("").catch(() => {}); // back to normal speakers
        await player.setLocalOutput("").catch(() => {});
        await invoke("eq_stop");
        this.enabled = false;
      }
      store(ON_KEY, this.enabled ? "1" : "0");
    } catch (e) {
      if (!quiet) toast.show(errMsg(e));
      await this.refresh();
    } finally {
      this.busy = false;
    }
  }

  setBand(i: number, db: number) {
    this.gains[i] = db;
    this.#push();
  }

  applyPreset(name: string) {
    this.gains = [...PRESETS[name]];
    this.#push();
  }

  #push() {
    store(KEY, JSON.stringify(this.gains));
    clearTimeout(this.#timer);
    this.#timer = setTimeout(() => invoke("eq_gains", { gains: this.gains }).catch(() => {}), 30);
  }
}

export const eq = new Equalizer();
