<script lang="ts">
  import "../app.css";
  let { children } = $props();
</script>

<div class="relative isolate min-h-dvh">
  <svg class="pointer-events-none fixed inset-0 -z-10 h-full w-full text-wave" viewBox="0 0 1440 1000" preserveAspectRatio="none" aria-hidden="true" focusable="false">
    <path fill="currentColor" opacity="0.65" d="M0 540 C240 445 345 680 600 590 S1110 180 1440 260 L1440 1000 H0 Z" />
    <path class="fill-wave-secondary" opacity="0.45" d="M0 750 C250 585 410 465 650 595 S1080 630 1440 490 L1440 1000 H0 Z" />
  </svg>
  {@render children()}
</div>
