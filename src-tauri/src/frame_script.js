// Injected into every frame (see lib.rs). Two jobs:
//  - In XSTIFY's own page: allow Spotify's player frame to choose an audio output.
//  - In Spotify's player frame (https://sdk.scdn.co): switch its audio output when XSTIFY
//    asks (used by the built-in equalizer to send the unprocessed sound to a silent output).
(() => {
  if (location.origin === "https://sdk.scdn.co") {
    let want = ""; // "" = the default output
    const els = new Set();
    const route = (el) => (el.setSinkId && el.sinkId !== want ? el.setSinkId(want).catch(() => {}) : null);

    const play = HTMLMediaElement.prototype.play;
    HTMLMediaElement.prototype.play = function () {
      els.add(this);
      const p = route(this);
      return p ? p.then(() => play.call(this)) : play.call(this);
    };

    window.addEventListener("message", async (e) => {
      const d = e.data;
      if (e.source !== window.parent || !d || d.xstify !== "sink") return;
      const reply = (msg) => window.parent.postMessage({ xstify: "sink-reply", id: d.id, ...msg }, "*");
      try {
        let id = "";
        if (d.label) {
          // Output device IDs are only revealed after mic permission; the mic is closed right away.
          const s = await navigator.mediaDevices.getUserMedia({ audio: true });
          s.getTracks().forEach((t) => t.stop());
          const outs = (await navigator.mediaDevices.enumerateDevices()).filter((x) => x.kind === "audiooutput");
          const dev = outs.find((x) => x.label === d.label) ?? outs.find((x) => x.label.includes(d.label));
          if (!dev) throw new Error(`Output "${d.label}" not found`);
          id = dev.deviceId;
        }
        want = id;
        document.querySelectorAll("audio,video").forEach((el) => els.add(el));
        await Promise.all([...els].map(route));
        reply({ ok: true });
      } catch (err) {
        reply({ ok: false, error: String(err?.message ?? err) });
      }
    });
  } else if (window === window.top) {
    // XSTIFY's page: give Spotify's player frame permission to pick an output.
    const extra = "microphone; speaker-selection";
    const desc = Object.getOwnPropertyDescriptor(HTMLIFrameElement.prototype, "allow");
    if (desc?.set) {
      Object.defineProperty(HTMLIFrameElement.prototype, "allow", {
        configurable: true,
        get() {
          return desc.get.call(this);
        },
        set(v) {
          desc.set.call(this, v && !String(v).includes("microphone") ? `${v}; ${extra}` : v || extra);
        },
      });
      const create = Document.prototype.createElement;
      Document.prototype.createElement = function (tag, opts) {
        const el = create.call(this, tag, opts);
        if (String(tag).toLowerCase() === "iframe") el.allow = "";
        return el;
      };
    }
  }
})();
