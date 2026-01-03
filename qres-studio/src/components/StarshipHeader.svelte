<script>
    // @ts-nocheck
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";

    let stats = {
        bytes_saved: 0,
        efficiency: 100,
        compressions: 0,
        active_nodes: 1,
    };

    async function loadStats() {
        try {
            const result = await invoke("load_stats");
            if (result) stats = result;
        } catch (e) {
            console.error("Stats load failed:", e);
        }
    }

    onMount(() => {
        loadStats();
        const interval = setInterval(loadStats, 5000);
        return () => clearInterval(interval);
    });
</script>

<header class="starship-header">
    <div class="brand">
        <div class="logo-orb"></div>
        <h1>QRES<span>STUDIO</span></h1>
        <span class="version">v8.2.0-AEON</span>
    </div>

    <div class="system-status">
        <div class="status-item">
            <span class="label">NODE STATUS</span>
            <span class="value online">ACTIVE</span>
        </div>
        <div class="status-item">
            <span class="label">COLLECTIVE</span>
            <span class="value">{stats.active_nodes} PEERS</span>
        </div>
        <div class="status-item">
            <span class="label">EFFICIENCY</span>
            <span class="value highlight">{stats.efficiency.toFixed(1)}%</span>
        </div>
        <div class="status-item">
            <span class="label">SAVED</span>
            <span class="value"
                >{(stats.bytes_saved / 1024 / 1024).toFixed(1)}MB</span
            >
        </div>
    </div>
</header>

<style>
    .starship-header {
        padding: 0.75rem 2rem;
        background: rgba(10, 10, 42, 0.9);
        backdrop-filter: blur(15px);
        border-bottom: 1px solid rgba(0, 255, 204, 0.3);
        display: flex;
        justify-content: space-between;
        align-items: center;
        z-index: 100;
        box-shadow: 0 5px 20px rgba(0, 0, 0, 0.4);
    }

    .brand {
        display: flex;
        align-items: center;
        gap: 1rem;
    }

    .logo-orb {
        width: 30px;
        height: 30px;
        background: radial-gradient(circle at 30% 30%, #00ffcc, #0080ff);
        border-radius: 50%;
        box-shadow: 0 0 15px rgba(0, 255, 204, 0.6);
        position: relative;
        overflow: hidden;
    }

    .logo-orb::after {
        content: "";
        position: absolute;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background: linear-gradient(rgba(255, 255, 255, 0.4), transparent);
        transform: rotate(-45deg);
    }

    h1 {
        margin: 0;
        font-size: 1.4rem;
        font-weight: 900;
        letter-spacing: 2px;
        color: #fff;
    }

    h1 span {
        font-weight: 300;
        color: #00ffcc;
    }

    .version {
        font-size: 0.7rem;
        color: rgba(255, 255, 255, 0.5);
        background: rgba(255, 255, 255, 0.1);
        padding: 2px 6px;
        border-radius: 4px;
        font-family: monospace;
    }

    .system-status {
        display: flex;
        gap: 2rem;
    }

    .status-item {
        display: flex;
        flex-direction: column;
        align-items: flex-end;
    }

    .label {
        font-size: 0.6rem;
        color: rgba(168, 218, 220, 0.6);
        letter-spacing: 1px;
        font-weight: 600;
    }

    .value {
        font-size: 0.9rem;
        font-weight: 700;
        color: #fff;
    }

    .value.online {
        color: #00ffcc;
        text-shadow: 0 0 10px rgba(0, 255, 204, 0.5);
    }

    .value.highlight {
        color: #0080ff;
        text-shadow: 0 0 10px rgba(0, 128, 255, 0.5);
    }
</style>
