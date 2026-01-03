<script lang="ts">
    // @ts-nocheck
    import { invoke } from "@tauri-apps/api/core";
    import { onMount, onDestroy } from "svelte";

    let peers: any[] = [];
    let canvas: HTMLCanvasElement;
    let ctx: CanvasRenderingContext2D;
    let animationId: number;
    let interval: ReturnType<typeof setInterval>;

    const NODE_RADIUS = 15;
    let width = 0;
    let height = 0;

    async function fetchPeers() {
        try {
            peers = await invoke("get_swarm_peers");
            // Randomly position for topology if not set
            peers.forEach((p, i) => {
                if (!p.vx) {
                    const angle = (2 * Math.PI * i) / peers.length;
                    p.x = width / 2 + Math.cos(angle) * 120;
                    p.y = height / 2 + Math.sin(angle) * 120;
                    p.vx = (Math.random() - 0.5) * 0.5;
                    p.vy = (Math.random() - 0.5) * 0.5;
                }
            });
        } catch (e) {
            console.error(e);
        }
    }

    function draw() {
        if (!ctx || !canvas) return;
        ctx.clearRect(0, 0, width, height);

        // Update positions
        peers.forEach((p) => {
            if (p.x === undefined) {
                p.x = Math.random() * width;
                p.y = Math.random() * height;
                p.vx = (Math.random() - 0.5) * 0.5;
                p.vy = (Math.random() - 0.5) * 0.5;
            }
            p.x += p.vx;
            p.y += p.vy;

            // Bounce
            if (p.x < 50 || p.x > width - 50) p.vx *= -1;
            if (p.y < 50 || p.y > height - 50) p.vy *= -1;
        });

        // Draw connections
        ctx.lineWidth = 1;
        peers.forEach((p1, i) => {
            peers.forEach((p2, j) => {
                if (i < j) {
                    const dist = Math.hypot(p1.x - p2.x, p1.y - p2.y);
                    if (dist < 200) {
                        const alpha = (1 - dist / 200) * 0.3;
                        ctx.strokeStyle = `rgba(0, 255, 204, ${alpha})`;
                        ctx.beginPath();
                        ctx.moveTo(p1.x, p1.y);
                        ctx.lineTo(p2.x, p2.y);
                        ctx.stroke();
                    }
                }
            });
        });

        // Draw nodes
        peers.forEach((p) => {
            const isLocal = p.id.includes("Local");

            // Glow
            ctx.shadowBlur = 10;
            ctx.shadowColor = isLocal ? "#00ffcc" : "#0080ff";

            ctx.fillStyle = isLocal ? "#00ffcc" : "#0080ff";
            ctx.beginPath();
            ctx.arc(p.x, p.y, NODE_RADIUS, 0, Math.PI * 2);
            ctx.fill();

            ctx.shadowBlur = 0;

            // Simple data orbs effect
            ctx.strokeStyle = "rgba(255, 255, 255, 0.2)";
            ctx.lineWidth = 2;
            ctx.beginPath();
            ctx.arc(p.x, p.y, NODE_RADIUS + 5, 0, Math.PI * 2);
            ctx.stroke();
        });

        animationId = requestAnimationFrame(draw);
    }

    onMount(() => {
        width = canvas.parentElement?.clientWidth || 800;
        height = canvas.parentElement?.clientHeight || 400;
        canvas.width = width;
        canvas.height = height;
        ctx = canvas.getContext("2d");
        fetchPeers();
        interval = setInterval(fetchPeers, 5000);
        draw();
    });

    onDestroy(() => {
        cancelAnimationFrame(animationId);
        clearInterval(interval);
    });
</script>

<div class="swarm-container">
    <div class="metrics-panel">
        <div class="chart-wrapper">
            <h4>Node Throughput (Mbps)</h4>
            <div class="placeholder-chart">
                <!-- Chart placeholder -->
                <div class="glow-ring"></div>
                <span>Chart Offline</span>
            </div>
        </div>
        <div class="stats-list">
            {#each peers as peer}
                <div class="peer-card" class:local={peer.id.includes("Local")}>
                    <span class="status-dot"></span>
                    <span class="peer-id">{peer.id}</span>
                    <span class="peer-meta"
                        >{peer.location} | {peer.latency_ms}ms</span
                    >
                </div>
            {/each}
        </div>
    </div>

    <div class="topology-panel">
        <h4>Neural Swarm Topology</h4>
        <canvas bind:this={canvas}></canvas>
    </div>
</div>

<style>
    .swarm-container {
        display: grid;
        grid-template-columns: 350px 1fr;
        gap: 1.5rem;
        height: 100%;
        padding: 1.5rem;
        background: transparent;
    }

    .metrics-panel {
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
        background: rgba(26, 26, 74, 0.4);
        padding: 1.5rem;
        border-radius: 12px;
        border: 1px solid rgba(0, 255, 204, 0.1);
    }

    .chart-wrapper {
        height: 300px;
        display: flex;
        flex-direction: column;
    }

    .placeholder-chart {
        flex: 1;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        position: relative;
        color: rgba(0, 255, 204, 0.5);
        font-size: 0.8rem;
    }

    .glow-ring {
        width: 150px;
        height: 150px;
        border: 2px dashed rgba(0, 255, 204, 0.2);
        border-radius: 50%;
        position: absolute;
        animation: rotate 20s linear infinite;
    }

    @keyframes rotate {
        from {
            transform: rotate(0deg);
        }
        to {
            transform: rotate(360deg);
        }
    }

    .topology-panel {
        background: rgba(26, 26, 74, 0.4);
        border-radius: 12px;
        border: 1px solid rgba(0, 255, 204, 0.1);
        display: flex;
        flex-direction: column;
        padding: 1rem;
    }

    h4 {
        margin: 0 0 1rem 0;
        color: #00ffcc;
        font-size: 0.9rem;
        text-transform: uppercase;
        letter-spacing: 1px;
    }

    canvas {
        flex: 1;
        width: 100%;
    }

    .stats-list {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
        overflow-y: auto;
    }

    .peer-card {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        padding: 0.75rem;
        background: rgba(0, 128, 255, 0.05);
        border: 1px solid rgba(0, 128, 255, 0.2);
        border-radius: 6px;
        font-size: 0.85rem;
    }

    .peer-card.local {
        border-color: rgba(0, 255, 204, 0.4);
        background: rgba(0, 255, 204, 0.05);
    }

    .status-dot {
        width: 8px;
        height: 8px;
        background: #00ffcc;
        border-radius: 50%;
        box-shadow: 0 0 5px #00ffcc;
    }

    .peer-id {
        color: #fff;
        font-weight: 500;
    }

    .peer-meta {
        margin-left: auto;
        color: #a8dadc;
        font-size: 0.75rem;
    }

    @media (max-width: 1024px) {
        .swarm-container {
            grid-template-columns: 1fr;
        }
    }
</style>
