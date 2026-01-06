<script>
    import { writable } from "svelte/store";
    import { invoke } from "@tauri-apps/api/core";
    import { SvelteToast } from "@zerodevx/svelte-toast";
    import { onMount } from "svelte";
    import { slide, fade } from "svelte/transition";
    import StarshipHeader from "./components/StarshipHeader.svelte";
    import StarshipSidebar from "./components/StarshipSidebar.svelte";
    import KnowledgeGraph from "./KnowledgeGraph.svelte";
    import SwarmView from "./SwarmView.svelte";
    import HiveMind from "./HiveMind.svelte";

    const graphData = writable({ nodes: [], edges: [] });
    let currentTab = "graph";
    let sidebarOpen = true;

    // Load initial data
    async function loadInitialData() {
        // @ts-ignore
        if (!window.__TAURI__) {
            return;
        }
        try {
            const stats = await invoke("load_stats");
            // Update stores
        } catch (e) {
            console.error("Failed to load stats:", e);
        }
    }

    onMount(() => {
        loadInitialData();
    });

    // Reactive graph updates
    function updateGraph() {
        // Fetch new graph data and update store
        // For now, static
    }
</script>

<SvelteToast />

<div class="starship-app">
    <StarshipHeader />

    <main class="dashboard-grid">
        <button
            class="sidebar-toggle"
            on:click={() => (sidebarOpen = !sidebarOpen)}>☰</button
        >
        {#if sidebarOpen}
            <aside class="sidebar" transition:slide={{ duration: 300 }}>
                <StarshipSidebar {updateGraph} />
            </aside>
        {/if}
        <section class="central-hologram">
            <nav class="orbital-tabs">
                <button
                    class:active={currentTab === "graph"}
                    on:click={() => (currentTab = "graph")}>Neural Graph</button
                >
                <button
                    class:active={currentTab === "swarm"}
                    on:click={() => (currentTab = "swarm")}>Swarm</button
                >
                <button
                    class:active={currentTab === "hive"}
                    on:click={() => (currentTab = "hive")}>Hive Mind</button
                >
            </nav>
            {#if currentTab === "graph"}
                <div class="holo-graph" transition:fade>
                    <KnowledgeGraph />
                </div>
            {:else if currentTab === "swarm"}
                <div class="gauge-cluster" transition:fade>
                    <SwarmView />
                </div>
            {:else if currentTab === "hive"}
                <div class="hive-view" transition:fade>
                    <HiveMind />
                </div>
            {/if}
        </section>
    </main>
</div>

<style>
    :global(body) {
        margin: 0;
        font-family:
            "Inter",
            -apple-system,
            BlinkMacSystemFont,
            "Segoe UI",
            sans-serif;
        overflow: hidden;
        background: #0a0a2a;
    }

    .starship-app {
        display: grid;
        grid-template-areas:
            "header header"
            "main main";
        grid-template-rows: auto 1fr;
        height: 100vh;
        color: #ffffff;
    }

    .dashboard-grid {
        grid-area: main;
        display: grid;
        grid-template-columns: auto 1fr;
        gap: 1rem;
        padding: 1rem;
        position: relative;
    }

    .sidebar-toggle {
        position: absolute;
        top: 1rem;
        left: 1rem;
        background: rgba(0, 128, 255, 0.1);
        border: 1px solid rgba(0, 128, 255, 0.3);
        border-radius: 50%;
        width: 3rem;
        height: 3rem;
        color: #00ffcc;
        cursor: pointer;
        z-index: 10;
        display: none; /* Hide on desktop */
    }

    .sidebar {
        width: 300px;
        background: rgba(0, 0, 0, 0.8);
        border: 1px solid rgba(0, 255, 204, 0.2);
        border-radius: 8px;
        padding: 1rem;
        box-shadow: 0 0 20px rgba(0, 255, 204, 0.1);
    }

    .central-hologram {
        position: relative;
        border: 1px solid rgba(0, 255, 204, 0.3);
        border-radius: 12px;
        box-shadow: 0 0 30px rgba(0, 255, 204, 0.2);
        background: rgba(10, 10, 42, 0.9);
        overflow: hidden;
    }

    .orbital-tabs {
        display: flex;
        justify-content: center;
        gap: 1rem;
        padding: 1rem;
        background: rgba(26, 26, 74, 0.6);
        border-bottom: 1px solid rgba(0, 255, 204, 0.2);
    }

    .orbital-tabs button {
        padding: 0.75rem 1.5rem;
        background: rgba(0, 128, 255, 0.1);
        border: 1px solid rgba(0, 128, 255, 0.3);
        border-radius: 20px;
        color: #a8dadc;
        cursor: pointer;
        transition: all 0.3s ease;
        font-size: 0.9rem;
        /* animation: orbit 10s linear infinite; */
    }

    .orbital-tabs button:hover {
        background: rgba(0, 128, 255, 0.2);
        border-color: #00ffcc;
        transform: scale(1.05);
    }

    .orbital-tabs button.active {
        background: linear-gradient(135deg, #00ffcc 0%, #0080ff 100%);
        border-color: #00ffcc;
        color: #0a0a2a;
        font-weight: 600;
        box-shadow: 0 0 15px rgba(0, 255, 204, 0.5);
    }

    .holo-graph,
    .gauge-cluster,
    .hive-view {
        height: calc(100vh - 200px);
        overflow: auto;
    }

    @keyframes orbit {
        from {
            transform: rotate(0deg);
        }
        to {
            transform: rotate(360deg);
        }
    }

    @media (max-width: 768px) {
        .dashboard-grid {
            grid-template-columns: 1fr;
        }
        .sidebar {
            width: 100%;
            margin-bottom: 1rem;
        }
        .sidebar-toggle {
            display: block;
        }
        .sidebar:not(.open) {
            display: none;
        }
    }
</style>
