<script lang="ts">
  // Loads the next page when scrolled into view.
  let { onmore }: { onmore: () => void } = $props();
  let el: HTMLDivElement;

  $effect(() => {
    const io = new IntersectionObserver((e) => e[0].isIntersecting && onmore(), { rootMargin: "400px" });
    io.observe(el);
    return () => io.disconnect();
  });
</script>

<div bind:this={el} style="height:1px"></div>
