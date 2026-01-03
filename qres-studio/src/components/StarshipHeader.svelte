<script>
    // @ts-nocheck
    import { invoke } from '@tauri-apps/api/core';
    import { onMount } from 'svelte';
    import { toast } from '@zerodevx/svelte-toast';

    let stats = { bytes_saved: 0, efficiency: 0, compressions: 0, active_nodes: 0 };

    async function loadStats() {
        // @ts-ignore
        if (!window.__TAURI__) {
            return;
        }
        try {
            stats = await invoke('load_stats');
        } catch (e) {
            console.error('Stats load failed:', e);
            toast.push(`Stats load failed: ${e}`);
        }
    }

    onMount(() => {
        loadStats();
        const interval = setInterval(loadStats, 5000);
        return () => clearInterval(interval);
    });
</script>

<header class="neon-header">
    <h1>QRES Studio v8.2</h1>
    <div class="stats-gauge">
        <span>Peers: {stats.active_nodes}</span>
        <span>Fidelity: {stats.efficiency.toFixed(1)}%</span>
        <span>Saved: {(stats.bytes_saved / 1024 / 1024).toFixed(1)}MB</span>
    </div>
</header>

<style>
    .neon-header {
        grid-area: header;
        padding: 1rem 2rem;
        background: linear-gradient(#0a0a2a, #1a1a4a);
        border-bottom: 1px solid rgba(0, 255, 204, 0.2);
        box-shadow: 0 0 20px rgba(0, 255, 204, 0.3);
        animation: pulse 2s infinite;
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    h1 {
        margin: 0;
        font-size: 1.5rem;
        font-weight: 700;
        background: linear-gradient(135deg, #00ffcc 0%, #0080ff 100%);
        background-clip: text;
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
    }

    .stats-gauge {
        display: flex;
        gap: 1rem;
        font-size: 0.9rem;
        color: #a8dadc;
    }

    .stats-gauge span {
        padding: 0.5rem 1rem;
        background: rgba(0, 128, 255, 0.1);
        border: 1px solid rgba(0, 128, 255, 0.3);
        border-radius: 20px;
        animation: glow 3s ease-in-out infinite alternate;
    }

    @keyframes pulse {
        0%, 100% { box-shadow: 0 0 20px rgba(0, 255, 204, 0.3); }
        50% { box-shadow: 0 0 30px rgba(0, 255, 204, 0.5); }
    }

    @keyframes glow {
        from { box-shadow: 0 0 5px rgba(0, 128, 255, 0.5); }
        to { box-shadow: 0 0 10px rgba(0, 128, 255, 0.8); }
    }
</style>
