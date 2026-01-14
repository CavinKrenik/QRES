<script lang="ts">
    import { onMount } from "svelte";
    import { currentPacket, streamingActive } from "../lib/iotStore";

    let canvas: HTMLCanvasElement;
    let ctx: CanvasRenderingContext2D;
    let neurons: Neuron[] = [];

    interface Neuron {
        x: number;
        y: number;
        energy: number;
    }

    // Initialize 50 random neurons
    function initNeurons() {
        neurons = Array(50)
            .fill(0)
            .map(() => ({
                x: Math.random() * 300,
                y: Math.random() * 200,
                energy: 0,
            }));
    }

    function draw() {
        if (!ctx || !canvas) return;

        // Cyberpunk trail effect
        ctx.fillStyle = "rgba(10, 10, 42, 0.2)";
        ctx.fillRect(0, 0, canvas.width, canvas.height);

        neurons.forEach((n) => {
            if (n.energy > 0.1) {
                ctx.beginPath();
                ctx.arc(n.x, n.y, n.energy * 3, 0, Math.PI * 2);
                ctx.fillStyle = `rgba(0, 255, 204, ${n.energy})`;
                ctx.fill();

                // Decay energy
                n.energy *= 0.9;
            }
        });

        // Random Synapse connections
        ctx.strokeStyle = "rgba(233, 69, 96, 0.3)"; // Magenta
        ctx.lineWidth = 0.5;
        ctx.beginPath();
        neurons.forEach((n, i) => {
            if (n.energy > 0.5) {
                // Connect to a random neighbor
                const target =
                    neurons[
                        (i + Math.floor(Math.random() * 5)) % neurons.length
                    ];
                ctx.moveTo(n.x, n.y);
                ctx.lineTo(target.x, target.y);
            }
        });
        ctx.stroke();

        requestAnimationFrame(draw);
    }

    // React to incoming data by firing neurons
    $: if ($currentPacket && $streamingActive) {
        // Activate random neurons based on vibration intensity
        const intensity = Math.min($currentPacket.vibration / 10, 1.0);
        const spikeCount = Math.floor(intensity * 10);

        for (let i = 0; i < spikeCount; i++) {
            const idx = Math.floor(Math.random() * neurons.length);
            neurons[idx].energy = 1.0;
        }
    }

    onMount(() => {
        ctx = canvas.getContext("2d")!;
        initNeurons();
        draw();
    });
</script>

<div class="brain-container">
    <h3>MetaBrain Activity</h3>
    <canvas bind:this={canvas} width="300" height="200"></canvas>
    <div class="scan-line"></div>
</div>

<style>
    .brain-container {
        position: relative;
        background: #000;
        border: 1px solid #333;
        overflow: hidden;
        border-radius: 4px;
        display: flex;
        flex-direction: column;
        align-items: center;
    }

    h3 {
        position: absolute;
        top: 5px;
        left: 10px;
        margin: 0;
        font-size: 0.7rem;
        color: #00ffcc;
        text-transform: uppercase;
        z-index: 10;
        pointer-events: none;
    }

    canvas {
        width: 100%;
        height: 100%;
    }

    .scan-line {
        position: absolute;
        top: 0;
        left: 0;
        width: 100%;
        height: 2px;
        background: rgba(0, 255, 204, 0.5);
        animation: scan 3s linear infinite;
        box-shadow: 0 0 10px rgba(0, 255, 204, 0.8);
    }

    @keyframes scan {
        0% {
            top: 0%;
        }
        100% {
            top: 100%;
        }
    }
</style>
