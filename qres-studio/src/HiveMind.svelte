<script lang="ts">
    import { onMount } from "svelte";
    import * as d3 from "d3";
    import { streamingActive } from "./lib/iotStore";
    import { fly } from "svelte/transition";

    // --- State ---
    let width = 800;
    let height = 600;
    let canvas: HTMLCanvasElement;
    let ctx: CanvasRenderingContext2D | null;
    let simulation: d3.Simulation<any, any>;
    let transform = d3.zoomIdentity;

    // Selected Node for HUD
    let selectedNode: any = null;

    // --- Mock Swarm Data (Enhanced) ---
    let nodes = [
        {
            id: "ROOT",
            group: 1,
            val: 30,
            label: "Ω ROOT",
            ip: "192.168.1.100",
            cpu: 12,
            ram: "32GB",
        },
        {
            id: "Jetson-01",
            group: 2,
            val: 15,
            label: "Jetson A",
            ip: "192.168.1.101",
            cpu: 45,
            ram: "8GB",
        },
        {
            id: "Jetson-02",
            group: 2,
            val: 15,
            label: "Jetson B",
            ip: "192.168.1.102",
            cpu: 32,
            ram: "8GB",
        },
        {
            id: "Pi-C1",
            group: 3,
            val: 8,
            label: "Pi Worker",
            ip: "192.168.1.105",
            cpu: 88,
            ram: "4GB",
        },
        {
            id: "Pi-C2",
            group: 3,
            val: 8,
            label: "Pi Worker",
            ip: "192.168.1.106",
            cpu: 65,
            ram: "4GB",
        },
        {
            id: "ESP-W1",
            group: 4,
            val: 5,
            label: "Sensor A",
            ip: "192.168.1.120",
            cpu: 10,
            ram: "512KB",
        },
        {
            id: "ESP-W2",
            group: 4,
            val: 5,
            label: "Sensor B",
            ip: "192.168.1.121",
            cpu: 12,
            ram: "512KB",
        },
        {
            id: "ESP-W3",
            group: 4,
            val: 5,
            label: "Sensor C",
            ip: "192.168.1.122",
            cpu: 0,
            ram: "512KB",
        },
    ];

    let links = [
        { source: "Jetson-01", target: "ROOT" },
        { source: "Jetson-02", target: "ROOT" },
        { source: "Pi-C1", target: "Jetson-01" },
        { source: "Pi-C2", target: "Jetson-01" },
        { source: "ESP-W1", target: "Pi-C1" },
        { source: "ESP-W2", target: "Pi-C1" },
        { source: "ESP-W3", target: "Jetson-02" },
    ];

    let particles: any[] = [];

    onMount(() => {
        ctx = canvas.getContext("2d");
        resize();
        window.addEventListener("resize", resize);

        // 1. Setup Simulation
        simulation = d3
            .forceSimulation(nodes as any)
            .force(
                "link",
                d3
                    .forceLink(links)
                    .id((d: any) => d.id)
                    .distance(100),
            )
            .force("charge", d3.forceManyBody().strength(-500))
            .force("center", d3.forceCenter(width / 2, height / 2))
            .force(
                "collide",
                d3.forceCollide().radius((d: any) => d.val + 10),
            );

        // 2. Setup Zoom & Drag
        const zoom = d3
            .zoom()
            .scaleExtent([0.1, 8])
            .on("zoom", (e) => {
                transform = e.transform;
                render();
            });

        d3.select(canvas)
            .call(zoom as any)
            .call(
                d3
                    .drag()
                    .subject(dragSubject)
                    .on("start", dragStarted)
                    .on("drag", dragged)
                    .on("end", dragEnded) as any,
            )
            .on("click", handleCanvasClick);

        // 3. Animation Loop
        const timer = d3.timer(() => {
            if ($streamingActive) generateTraffic();
            updateParticles();
            render();
        });

        return () => {
            timer.stop();
            window.removeEventListener("resize", resize);
            simulation.stop();
        };
    });

    function generateTraffic() {
        if (Math.random() > 0.1) return;
        const link: any = links[Math.floor(Math.random() * links.length)];
        const reverse = Math.random() > 0.5;
        particles.push({
            source: reverse ? link.target : link.source,
            target: reverse ? link.source : link.target,
            progress: 0,
            speed: 0.02 + Math.random() * 0.02,
            color: reverse ? "#00ffcc" : "#ff4444",
        });
    }

    function updateParticles() {
        for (let i = particles.length - 1; i >= 0; i--) {
            let p = particles[i];
            p.progress += p.speed;
            if (p.progress >= 1) particles.splice(i, 1);
        }
    }

    function resize() {
        if (canvas && canvas.parentElement) {
            width = canvas.parentElement.clientWidth;
            height = canvas.parentElement.clientHeight;
            canvas.width = width;
            canvas.height = height;
            if (simulation) {
                simulation.force(
                    "center",
                    d3.forceCenter(width / 2, height / 2),
                );
                simulation.alpha(1).restart();
            }
        }
    }

    function render() {
        if (!ctx) return;

        ctx.save();
        ctx.clearRect(0, 0, width, height);
        ctx.translate(transform.x, transform.y);
        ctx.scale(transform.k, transform.k);

        // Draw Links
        ctx.strokeStyle = "rgba(100, 100, 100, 0.3)";
        ctx.lineWidth = 1;
        links.forEach((link: any) => {
            ctx!.beginPath();
            ctx!.moveTo(link.source.x, link.source.y);
            ctx!.lineTo(link.target.x, link.target.y);
            ctx!.stroke();
        });

        // Draw Particles
        particles.forEach((p) => {
            const x = p.source.x + (p.target.x - p.source.x) * p.progress;
            const y = p.source.y + (p.target.y - p.source.y) * p.progress;
            ctx!.beginPath();
            ctx!.fillStyle = p.color;
            ctx!.shadowBlur = 5;
            ctx!.shadowColor = p.color;
            ctx!.arc(x, y, 3 / transform.k, 0, 2 * Math.PI);
            ctx!.fill();
            ctx!.shadowBlur = 0;
        });

        // Draw Nodes
        nodes.forEach((node: any) => {
            ctx!.beginPath();

            const isSelected = selectedNode && selectedNode.id === node.id;
            const baseSize = node.val;

            if (isSelected) {
                ctx!.shadowBlur = 20;
                ctx!.shadowColor = "#fff";
            }

            let color = "#4488ff";
            if (node.group === 1) color = "#ff4444";
            if (node.group === 2) color = "#00ffcc";

            ctx!.fillStyle = color;
            ctx!.arc(node.x, node.y, baseSize, 0, 2 * Math.PI);
            ctx!.fill();
            ctx!.shadowBlur = 0;

            if (transform.k > 0.8 || node.group === 1) {
                ctx!.fillStyle = "#fff";
                ctx!.font = `${10 / transform.k}px JetBrains Mono`;
                ctx!.fillText(node.label, node.x + baseSize + 2, node.y + 4);
            }
        });

        ctx.restore();
    }

    function dragSubject(event: any) {
        const x = transform.invertX(event.x);
        const y = transform.invertY(event.y);
        return simulation.find(x, y, 30);
    }

    function handleCanvasClick(event: any) {
        const [x, y] = d3.pointer(event);
        const graphX = transform.invertX(x);
        const graphY = transform.invertY(y);
        const clickedNode = simulation.find(graphX, graphY, 30);
        selectedNode = clickedNode || null;
        render();
    }

    function dragStarted(event: any) {
        if (!event.active) simulation.alphaTarget(0.3).restart();
        event.subject.fx = event.subject.x;
        event.subject.fy = event.subject.y;
        selectedNode = event.subject;
    }

    function dragged(event: any) {
        event.subject.fx = event.x;
        event.subject.fy = event.y;
    }

    function dragEnded(event: any) {
        if (!event.active) simulation.alphaTarget(0);
        event.subject.fx = null;
        event.subject.fy = null;
    }
</script>

<div class="hive-container">
    <div class="overlay">
        <h2>Global Swarm State</h2>
        <div class="stat">NODES: <span>{nodes.length}</span></div>
        <div class="stat">ZOOM: <span>{transform.k.toFixed(1)}x</span></div>
        <div class="stat">
            STATUS: <span class:live={$streamingActive}
                >{$streamingActive ? "SYNCING" : "IDLE"}</span
            >
        </div>
    </div>

    <canvas bind:this={canvas}></canvas>

    {#if selectedNode}
        <div class="node-hud" transition:fly={{ x: 20, duration: 300 }}>
            <div class="hud-header">
                <h3>{selectedNode.label}</h3>
                <span class="badge" class:root={selectedNode.group === 1}>
                    {selectedNode.group === 1 ? "CONTROLLER" : "WORKER"}
                </span>
            </div>
            <div class="hud-grid">
                <div class="hud-item">
                    <span class="hud-label">IP ADDRESS</span>
                    <span>{selectedNode.ip}</span>
                </div>
                <div class="hud-item">
                    <span class="hud-label">CPU LOAD</span>
                    <div class="bar-container">
                        <div
                            class="bar"
                            style="width: {selectedNode.cpu}%"
                        ></div>
                    </div>
                    <span class="val">{selectedNode.cpu}%</span>
                </div>
                <div class="hud-item">
                    <span class="hud-label">MEMORY</span>
                    <span>{selectedNode.ram}</span>
                </div>
                <div class="hud-item">
                    <span class="hud-label">STATUS</span>
                    <span class="status-ok">● ONLINE</span>
                </div>
            </div>
            <button class="action-btn" on:click={() => (selectedNode = null)}
                >CLOSE</button
            >
        </div>
    {/if}
</div>

<style>
    .hive-container {
        width: 100%;
        height: 100%;
        background: #050510;
        position: relative;
        overflow: hidden;
    }

    canvas {
        display: block;
        cursor: crosshair;
    }

    .overlay {
        position: absolute;
        top: 20px;
        left: 20px;
        pointer-events: none;
        color: #fff;
    }

    h2 {
        margin: 0 0 10px 0;
        font-size: 1rem;
        color: #00ffcc;
        text-shadow: 0 0 10px rgba(0, 255, 204, 0.5);
    }

    .stat {
        font-family: "JetBrains Mono", monospace;
        color: #888;
        font-size: 0.8rem;
        margin-bottom: 4px;
    }
    .stat span {
        color: #eee;
    }
    .stat span.live {
        color: #00ffcc;
    }

    .node-hud {
        position: absolute;
        top: 20px;
        right: 20px;
        width: 260px;
        background: rgba(10, 15, 30, 0.95);
        border: 1px solid #00ffcc;
        backdrop-filter: blur(10px);
        padding: 1rem;
        box-shadow: 0 0 30px rgba(0, 0, 0, 0.8);
        border-radius: 4px;
        color: #fff;
    }

    .hud-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        border-bottom: 1px solid rgba(255, 255, 255, 0.1);
        padding-bottom: 0.5rem;
        margin-bottom: 1rem;
    }

    .hud-header h3 {
        margin: 0;
        color: #eee;
        font-size: 1rem;
    }

    .badge {
        font-size: 0.6rem;
        padding: 2px 6px;
        background: #333;
        border-radius: 2px;
        color: #888;
    }
    .badge.root {
        background: #ff4444;
        color: #000;
    }

    .hud-grid {
        display: flex;
        flex-direction: column;
        gap: 0.8rem;
    }

    .hud-item {
        display: flex;
        flex-direction: column;
        gap: 2px;
    }

    .hud-label {
        font-size: 0.6rem;
        color: #666;
        letter-spacing: 1px;
    }

    .hud-item span {
        font-family: "JetBrains Mono", monospace;
        font-size: 0.9rem;
    }

    .status-ok {
        color: #00ffcc;
    }

    .bar-container {
        width: 100%;
        height: 4px;
        background: #222;
        margin-top: 4px;
    }
    .bar {
        height: 100%;
        background: #00ffcc;
        box-shadow: 0 0 10px #00ffcc;
    }

    .action-btn {
        width: 100%;
        margin-top: 1rem;
        background: transparent;
        border: 1px solid #444;
        color: #888;
        padding: 0.5rem;
        cursor: pointer;
        font-family: "JetBrains Mono", monospace;
        font-size: 0.7rem;
        transition: all 0.2s;
    }
    .action-btn:hover {
        border-color: #fff;
        color: #fff;
    }
</style>
