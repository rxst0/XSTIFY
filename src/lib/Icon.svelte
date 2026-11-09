<script lang="ts">
  const paths: Record<string, string> = {
    play: "M8 5v14l11-7z",
    pause: "M6 5h4v14H6zM14 5h4v14h-4z",
    next: "M6 6l8.5 6L6 18zM16 6h2v12h-2z",
    prev: "M18 6l-8.5 6L18 18zM6 6h2v12H6z",
    search: "M10 3a7 7 0 1 0 4.2 12.6l5.1 5.1 1.4-1.4-5.1-5.1A7 7 0 0 0 10 3zm0 2a5 5 0 1 1 0 10 5 5 0 0 1 0-10z",
    heart: "M12 21s-7.5-4.6-9.6-9.3C.9 8.4 3 4.5 6.8 4.5c2.1 0 3.6 1.2 5.2 3 1.6-1.8 3.1-3 5.2-3 3.8 0 5.9 3.9 4.4 7.2C19.5 16.4 12 21 12 21z",
    back: "M15.4 5.4L14 4l-8 8 8 8 1.4-1.4L8.8 12z",
    volume: "M4 9v6h4l5 4V5L8 9zM16 8.5a4.5 4.5 0 0 1 0 7v-7z",
    mute: "M4 9v6h4l5 4V5L8 9zM16.5 9.5l1.4-1.4L20 10.2l2.1-2.1 1.4 1.4-2.1 2.1 2.1 2.1-1.4 1.4-2.1-2.1-2.1 2.1-1.4-1.4 2.1-2.1z",
    settings: "M19.4 13a7.6 7.6 0 0 0 0-2l2.1-1.6-2-3.5-2.5 1a7.4 7.4 0 0 0-1.7-1L15 3.3h-4l-.4 2.6a7.4 7.4 0 0 0-1.7 1l-2.5-1-2 3.5L6.6 11a7.6 7.6 0 0 0 0 2l-2.1 1.6 2 3.5 2.5-1a7.4 7.4 0 0 0 1.7 1l.4 2.6h4l.4-2.6a7.4 7.4 0 0 0 1.7-1l2.5 1 2-3.5zM13 15.5a3.5 3.5 0 1 1 0-7 3.5 3.5 0 0 1 0 7z",
    bass:"M3 10h2v4H3zM7 6h2v12H7zM11 8h2v8h-2zM15 3h2v18h-2zM19 10h2v4h-2z",
    music:"M9 3v10.6A4 4 0 1 0 11 17V7h6V3z",
    logout: "M10 3H4v18h6v-2H6V5h4zM16 7l-1.4 1.4 2.6 2.6H9v2h8.2l-2.6 2.6L16 17l5-5z",
  };
  let { name, size = 20 }: { name: string; size?: number } = $props();
</script>

<svg width={size} height={size} viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d={paths[name]} /></svg>
