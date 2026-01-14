<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import * as d3 from "d3";
    import { streamingActive, currentPacket } from "./lib/iotStore";

    // --- Config ---
    let width = 800;
    let height = 600;
    let canvas: HTMLCanvasElement;
    let ctx: CanvasRenderingContext2D | null;
    let animationId: number;

    // --- Neural Architecture ---
    const layers = [
        { id: 0, nodes: 4, label: "INPUT SENSORS" },
        { id: 1, nodes: 8, label: "HIDDEN LAYER A" },
        { id: 2, nodes: 8, label: "HIDDEN LAYER B" },
        { id: 3, nodes: 6, label: "ATTENTION HEAD" },
        { id: 4, nodes: 2, label: "OUTPUT (AQC)" },
    ];

    let neurons: any[] = [];
    let synapses: any[] = [];
    let spikes: any[] = [];

    onMount(() => {
        ctx = canvas.getContext("2d");
        resize();
        window.addEventListener("resize", resize);
        buildNetwork();
        animate();

        return () => {
            window.removeEventListener("resize", resize);
            cancelAnimationFrame(animationId);
        };
    });

    $: if ($currentPacket && $streamingActive) {
        triggerPulse($currentPacket);
    }

    function buildNetwork() {
        neurons = [];
        synapses = [];

        const layerWidth = width / (layers.length + 1);

        layers.forEach((layer, lIdx) => {
            const x = (lIdx + 1) * layerWidth;
            const spacing = height / (layer.nodes + 1);

            for (let i = 0; i < layer.nodes; i++) {
                neurons.push({
                    id: `L${lIdx}-N${i}`,
                    layer: lIdx,
                    x: x,
                    y: (i + 1) * spacing,
                    val: 0,
                    label: getLabel(lIdx, i),
                });
            }
        });

        for (let l = 0; l < layers.length - 1; l++) {
            const sourceLayer = neurons.filter((n) => n.layer === l);
            const targetLayer = neurons.filter((n) => n.layer === l + 1);

            sourceLayer.forEach((src) => {
                targetLayer.forEach((tgt) => {
                    if (Math.random() > 0.3) {
                        synapses.push({
                            source: src,
                            target: tgt,
                            weight: Math.random(),
                        });
                    }
                });
            });
        }
    }

    function getLabel(layer: number, idx: number) {
        if (layer === 0) {
            const labels = ["TEMP", "VIBRA", "BATT", "TIME"];
            return labels[idx] || "IN";
        }
        if (layer === 4) return idx === 0 ? "RATIO" : "MODE";
        return "";
    }

    function triggerPulse(packet: any) {
        const inputs = neurons.filter((n) => n.layer === 0);
        if (inputs.length >= 2) {
            inputs[0].val = 1.0;
            inputs[1].val = packet.vibration > 0.5 ? 1.0 : 0.2;
        }

        for (let i = 0; i < 5; i++) {
            const startNode = inputs[Math.floor(Math.random() * inputs.length)];
            spikes.push({
                x: startNode.x,
                y: startNode.y,
                targetLayer: 1,
                progress: 0,
                speed: 0.05 + Math.random() * 0.05,
                path: [],
            });
        }
    }

    function resize() {
        if (canvas && canvas.parentElement) {
            width = canvas.parentElement.clientWidth;
            height = canvas.parentElement.clientHeight;
            canvas.width = width;
            canvas.height = height;
            buildNetwork();
        }
    }

    function animate() {
        if (!ctx) return;

        ctx.fillStyle = "rgba(5, 5, 16, 0.2)";
        ctx.fillRect(0, 0, width, height);

        ctx.lineWidth = 1;
        synapses.forEach((syn) => {
            ctx!.beginPath();
            ctx!.moveTo(syn.source.x, syn.source.y);
            ctx!.lineTo(syn.target.x, syn.target.y);
            ctx!.strokeStyle = `rgba(0, 255, 204, ${syn.weight * 0.15})`;
            ctx!.stroke();
        });

        for (let i = spikes.length - 1; i >= 0; i--) {
            let s = spikes[i];
            s.progress += s.speed;

            const layerWidth = width / (layers.length + 1);
            const currentX = s.x + s.speed * layerWidth * 5;
            s.x = currentX;

            ctx.beginPath();
            ctx.fillStyle = "#fff";
            ctx.shadowBlur = 10;
            ctx.shadowColor = "#00ffcc";
            ctx.arc(s.x, s.y, 2, 0, 2 * Math.PI);
            ctx.fill();
            ctx.shadowBlur = 0;

            if (s.progress >= 1.0) {
                const targetNodes = neurons.filter(
                    (n) => n.layer === s.targetLayer,
                );
                if (targetNodes.length > 0) {
                    const nextNode =
                        targetNodes[
                            Math.floor(Math.random() * targetNodes.length)
                        ];
                    nextNode.val = 1.0;
                    s.y = nextNode.y;

                    if (s.targetLayer < layers.length - 1) {
                        s.targetLayer++;
                        s.progress = 0;
                    } else {
                        spikes.splice(i, 1);
                    }
                } else {
                    spikes.splice(i, 1);
                }
            }
        }

        neurons.forEach((node) => {
            node.val *= 0.92;

            ctx!.beginPath();
            const radius = 6 + node.val * 4;

            let color = "rgba(0, 255, 204, 0.5)";
            if (node.layer === 0) color = "rgba(255, 68, 68, 0.8)";
            if (node.layer === layers.length - 1)
                color = "rgba(68, 136, 255, 0.8)";

            if (node.val > 0.1) {
                ctx!.fillStyle = "#fff";
                ctx!.shadowBlur = 15;
                ctx!.shadowColor = color;
            } else {
                ctx!.fillStyle = color;
                ctx!.shadowBlur = 0;
            }

            ctx!.arc(node.x, node.y, radius, 0, 2 * Math.PI);
            ctx!.fill();

            if (node.label) {
                ctx!.fillStyle = "#888";
                ctx!.font = "10px JetBrains Mono";
                ctx!.fillText(node.label, node.x - 20, node.y - 15);
            }
        });

        layers.forEach((l, i) => {
            const x = (i + 1) * (width / (layers.length + 1));
            ctx!.fillStyle = "rgba(255,255,255,0.1)";
            ctx!.font = "11px JetBrains Mono";
            ctx!.fillText(l.label, x - 50, height - 20);
        });

        animationId = requestAnimationFrame(animate);
    }
</script>

<div class="neural-container">
    <div class="overlay">
        <h2>MetaBrain™ Topology</h2>
        <div class="metric">
            LAYERS: <span class="val">5 (DEEP)</span>
        </div>
        <div class="metric">
            SYNAPSES: <span class="val">{synapses.length}</span>
        </div>
        <div class="metric">
            ACTIVITY: <span class="val" class:active={$streamingActive}>
                {$streamingActive ? "SPIKING" : "DORMANT"}
            </span>
        </div>
    </div>
    <canvas bind:this={canvas}></canvas>
</div>

<style>
    .neural-container {
        width: 100%;
        height: 100%;
        background: #02020a;
        position: relative;
        overflow: hidden;
    }

    canvas {
        display: block;
    }

    .overlay {
        position: absolute;
        top: 20px;
        left: 20px;
        pointer-events: none;
    }

    h2 {
        margin: 0 0 1rem 0;
        color: #00ffcc;
        font-size: 1.1rem;
        letter-spacing: 2px;
        text-shadow: 0 0 10px rgba(0, 255, 204, 0.4);
    }

    .metric {
        font-family: "JetBrains Mono", monospace;
        color: #666;
        font-size: 0.8rem;
        margin-bottom: 0.5rem;
    }

    .val {
        color: #eee;
        margin-left: 0.5rem;
    }

    .val.active {
        color: #ff4444;
        font-weight: bold;
        text-shadow: 0 0 8px #ff4444;
    }
</style>
