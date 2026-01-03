<script>
    import { invoke } from "@tauri-apps/api/core";
    import DropZone from "./DropZone.svelte";
    import HiveMind from "./HiveMind.svelte";
    import SwarmView from "./SwarmView.svelte";
    import KnowledgeGraph from "./KnowledgeGraph.svelte";
    import Controls from "./components/Controls.svelte";
    import SwarmDashboard from "./components/SwarmDashboard.svelte";

    let activeTab = "compress";
    let stats = { total_compressions: 0, bytes_saved: 0 };

    async function loadStats() {
        try {
            stats = await invoke("get_stats");
        } catch (e) {
            console.error("Failed to load stats:", e);
        }
    }

    loadStats();
</script>

<main>
    <div class="app-header">
        <h1>QRES Studio v8.1</h1>
        <div class="stats-bar">
            <span
                >💾 Saved: {(stats.bytes_saved / 1024 / 1024).toFixed(
                    1,
                )}MB</span
            >
            <span>📦 Files: {stats.total_compressions}</span>
        </div>
    </div>

    <div class="tabs">
        <button
            class:active={activeTab === "compress"}
            on:click={() => (activeTab = "compress")}
        >
            Drop Zone
        </button>
        <button
            class:active={activeTab === "controls"}
            on:click={() => (activeTab = "controls")}
        >
            🎛️ Controls
        </button>
        <button
            class:active={activeTab === "swarm"}
            on:click={() => (activeTab = "swarm")}
        >
            🌐 Swarm
        </button>
        <button
            class:active={activeTab === "neural"}
            on:click={() => (activeTab = "neural")}
        >
            Neural Graph
        </button>
        <button
            class:active={activeTab === "hive"}
            on:click={() => (activeTab = "hive")}
        >
            Hive Mainnet
        </button>
        <button
            class:active={activeTab === "visual"}
            on:click={() => (activeTab = "visual")}
        >
            Swarm Topology
        </button>
    </div>

    <div class="tab-content">
        {#if activeTab === "compress"}
            <DropZone on:complete={loadStats} />
        {:else if activeTab === "controls"}
            <Controls />
        {:else if activeTab === "swarm"}
            <SwarmDashboard />
        {:else if activeTab === "neural"}
            <KnowledgeGraph />
        {:else if activeTab === "hive"}
            <HiveMind />
        {:else if activeTab === "visual"}
            <SwarmView />
        {/if}
    </div>
</main>

<style>
    :global(body) {
        margin: 0;
        font-family:
            "Inter",
            -apple-system,
            BlinkMacSystemFont,
            "Segoe UI",
            sans-serif;
        background: linear-gradient(135deg, #0a0e27 0%, #1a1f3a 100%);
        color: #e0e7ff;
        overflow: hidden;
    }

    main {
        height: 100vh;
        display: flex;
        flex-direction: column;
    }

    .app-header {
        padding: 1.5rem 2rem;
        background: rgba(15, 23, 42, 0.8);
        backdrop-filter: blur(10px);
        border-bottom: 1px solid rgba(99, 102, 241, 0.2);
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    h1 {
        margin: 0;
        font-size: 1.5rem;
        font-weight: 700;
        background: linear-gradient(135deg, #818cf8 0%, #c084fc 100%);
        background-clip: text;
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
    }

    .stats-bar {
        display: flex;
        gap: 2rem;
        font-size: 0.9rem;
        color: #94a3b8;
    }

    .tabs {
        display: flex;
        padding: 0 2rem;
        gap: 0.5rem;
        background: rgba(15, 23, 42, 0.6);
        border-bottom: 1px solid rgba(99, 102, 241, 0.2);
    }

    .tabs button {
        padding: 1rem 2rem;
        background: transparent;
        border: none;
        color: #94a3b8;
        cursor: pointer;
        border-bottom: 2px solid transparent;
        transition: all 0.3s;
        font-size: 0.95rem;
        font-weight: 500;
    }

    .tabs button:hover {
        color: #c7d2fe;
        background: rgba(99, 102, 241, 0.1);
    }

    .tabs button.active {
        color: #818cf8;
        border-bottom-color: #818cf8;
    }

    .tab-content {
        flex: 1;
        overflow: auto;
    }
</style>
