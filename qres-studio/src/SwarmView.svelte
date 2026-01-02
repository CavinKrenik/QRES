<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { onMount, onDestroy } from "svelte";

    let peers: any[] = [];
    let canvas: HTMLCanvasElement;
    let ctx: CanvasRenderingContext2D;
    let animationId: number;
    let interval: ReturnType<typeof setInterval>;

    const NODE_RADIUS = 20;
    const CENTER_X = 400;
    const CENTER_Y = 300;

    async function fetchPeers() {
        try {
            peers = await invoke("get_swarm_peers");
            // Add visual positions if not set (simple circular layout)
            peers.forEach((p, i) => {
                const angle = (2 * Math.PI * i) / peers.length;
                // Random drift
                p.x =
                    CENTER_X +
                    Math.cos(angle) * 150 +
                    (Math.random() - 0.5) * 20;
                p.y =
                    CENTER_Y +
                    Math.sin(angle) * 150 +
                    (Math.random() - 0.5) * 20;
                // Signal activity pulse
                p.pulse = Math.random();
            });
        } catch (e) {
            console.error(e);
        }
    }

    function draw() {
        if (!ctx) return;
        ctx.clearRect(0, 0, canvas.width, canvas.height);

        // Draw Connections
        ctx.lineWidth = 1;
        peers.forEach((p1, i) => {
            peers.forEach((p2, j) => {
                if (i < j) {
                    const dist = Math.hypot(p1.x - p2.x, p1.y - p2.y);
                    if (dist < 250) {
                        const alpha = 1 - dist / 250;
                        ctx.strokeStyle = `rgba(129, 140, 248, ${alpha * 0.5})`;
                        ctx.beginPath();
                        ctx.moveTo(p1.x, p1.y);
                        ctx.lineTo(p2.x, p2.y);
                        ctx.stroke();

                        // Traffic Particles
                        if (Date.now() % 20 === 0 && Math.random() > 0.8) {
                            drawPacket(p1, p2);
                        }
                    }
                }
            });
        });

        // Draw Nodes
        peers.forEach((p) => {
            // Pulse Effect
            p.pulse += 0.05;
            const r = NODE_RADIUS + Math.sin(p.pulse) * 2;

            ctx.fillStyle = p.id.includes("Local") ? "#10b981" : "#818cf8";
            ctx.beginPath();
            ctx.arc(p.x, p.y, r, 0, Math.PI * 2);
            ctx.fill();

            // Label
            ctx.fillStyle = "#e0e7ff";
            ctx.font = "12px Inter";
            ctx.textAlign = "center";
            ctx.fillText(p.id, p.x, p.y + 35);
            ctx.fillStyle = "#94a3b8";
            ctx.font = "10px Inter";
            ctx.fillText(`${p.latency_ms}ms`, p.x, p.y + 48);
        });

        animationId = requestAnimationFrame(draw);
    }

    function drawPacket(p1: any, p2: any) {
        // Simplified visual effect, ideally we track packet animations
    }

    onMount(() => {
        ctx = canvas.getContext("2d")!;
        fetchPeers();
        interval = setInterval(fetchPeers, 3000);
        draw();
    });

    onDestroy(() => {
        cancelAnimationFrame(animationId);
        clearInterval(interval);
    });
</script>

<div class="swarm-view">
    <div class="overlay">
        <h3>Live Swarm Topology</h3>
        <p>Active Nodes: {peers.length}</p>
    </div>
    <canvas bind:this={canvas} width="800" height="600"></canvas>
</div>

<style>
    .swarm-view {
        position: relative;
        background: radial-gradient(circle at center, #1e293b 0%, #0f172a 100%);
        border-radius: 12px;
        overflow: hidden;
        height: 600px;
        box-shadow: inset 0 0 20px rgba(0, 0, 0, 0.5);
    }

    .overlay {
        position: absolute;
        top: 20px;
        left: 20px;
        pointer-events: none;
    }

    h3 {
        margin: 0;
        color: #818cf8;
    }

    p {
        color: #94a3b8;
        font-size: 0.9rem;
    }

    canvas {
        width: 100%;
        height: 100%;
    }
</style>
