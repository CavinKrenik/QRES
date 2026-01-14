<script>
    // @ts-nocheck
    import { onMount } from "svelte";

    let stars = [];

    onMount(() => {
        // Generate random stars
        for (let i = 0; i < 100; i++) {
            stars.push({
                x: Math.random() * 100,
                y: Math.random() * 100,
                size: Math.random() * 2 + 1,
                duration: Math.random() * 3 + 2,
            });
        }
        stars = stars; // Trigger reactivity
    });
</script>

<div class="starfield">
    {#each stars as star}
        <div
            class="star"
            style="left: {star.x}%; top: {star.y}%; width: {star.size}px; height: {star.size}px; animation-duration: {star.duration}s;"
        ></div>
    {/each}
</div>

<style>
    .starfield {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background: radial-gradient(
            ellipse at bottom,
            #1a1a4a 0%,
            #0a0a2a 100%
        );
        overflow: hidden;
        z-index: -1;
    }

    .star {
        position: absolute;
        background: #ffffff;
        border-radius: 50%;
        opacity: 0.8;
        animation: twinkle infinite ease-in-out;
    }

    @keyframes twinkle {
        0%,
        100% {
            opacity: 0.3;
        }
        50% {
            opacity: 1;
        }
    }
</style>
