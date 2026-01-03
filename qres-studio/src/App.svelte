<script>
    // @ts-nocheck
    import { writable } from "svelte/store";
    import { invoke } from "@tauri-apps/api/core";
    import { fade, slide } from "svelte/transition";
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
        try {
            const stats = await invoke("load_stats");
            // Update stores
        } catch (e) {
            console.error("Failed to load stats:", e);
        }
    }

    // Reactive graph updates
    function updateGraph() {
        // This will be called after compression to refresh the graph
    }
</script>

<!-- <Toaster position="top-right" /> -->

<div class="starship-app">
    <StarshipHeader />

    <main class="dashboard-grid">
        <button
            class="sidebar-toggle"
            on:click={() => (sidebarOpen = !sidebarOpen)}
            title="Toggle Sidebar"
        >
            {sidebarOpen ? "◀" : "▶"}
        </button>

        {#if sidebarOpen}
            <aside
                class="sidebar"
                transition:slide={{ axis: "x", duration: 400 }}
            >
                <StarshipSidebar {updateGraph} />
            </aside>
        {/if}

        <section class="central-hologram">
            <nav class="orbital-tabs">
                <button
                    class:active={currentTab === "graph"}
                    on:click={() => (currentTab = "graph")}
                    class="tab-btn"
                >
                    Neural Graph
                </button>
                <button
                    class:active={currentTab === "swarm"}
                    on:click={() => (currentTab = "swarm")}
                    class="tab-btn"
                >
                    Live Swarm
                </button>
                <button
                    class:active={currentTab === "hive"}
                    on:click={() => (currentTab = "hive")}
                    class="tab-btn"
                >
                    Hive Stats
                </button>
            </nav>

            <div class="content-viewport">
                {#if currentTab === "graph"}
                    <div class="view-pane" transition:fade>
                        <KnowledgeGraph />
                    </div>
                {:else if currentTab === "swarm"}
                    <div class="view-pane" transition:fade>
                        <SwarmView />
                    </div>
                {:else if currentTab === "hive"}
                    <div class="view-pane" transition:fade>
                        <HiveMind />
                    </div>
                {/if}
            </div>
        </section>
    </main>
</div>

<style>
    :global(body) {
        margin: 0;
        font-family:
            "Outfit",
            "Inter",
            -apple-system,
            sans-serif;
        overflow: hidden;
        background: #05051a;
        color: #fff;
    }

    .starship-app {
        display: flex;
        flex-direction: column;
        height: 100vh;
        z-index: 1;
        position: relative;
    }

    .dashboard-grid {
        flex: 1;
        display: grid;
        grid-template-columns: auto 1fr;
        gap: 0;
        overflow: hidden;
        position: relative;
    }

    .sidebar-toggle {
        position: absolute;
        bottom: 2rem;
        left: 1rem;
        background: rgba(0, 255, 204, 0.1);
        border: 1px solid rgba(0, 255, 204, 0.4);
        color: #00ffcc;
        width: 2.5rem;
        height: 2.5rem;
        border-radius: 50%;
        cursor: pointer;
        z-index: 100;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: all 0.3s;
        box-shadow: 0 0 10px rgba(0, 255, 204, 0.2);
    }

    .sidebar-toggle:hover {
        background: rgba(0, 255, 204, 0.2);
        box-shadow: 0 0 20px rgba(0, 255, 204, 0.4);
        transform: scale(1.1);
    }

    .sidebar {
        width: 320px;
        height: 100%;
        background: rgba(10, 10, 42, 0.8);
        backdrop-filter: blur(10px);
        border-right: 1px solid rgba(0, 255, 204, 0.2);
        box-shadow: 10px 0 30px rgba(0, 0, 0, 0.5);
    }

    .central-hologram {
        position: relative;
        display: flex;
        flex-direction: column;
        overflow: hidden;
        background: radial-gradient(
            circle at center,
            rgba(0, 80, 255, 0.05) 0%,
            transparent 70%
        );
    }

    .orbital-tabs {
        display: flex;
        justify-content: center;
        gap: 2rem;
        padding: 1.5rem;
        z-index: 10;
    }

    .tab-btn {
        padding: 0.8rem 1.5rem;
        background: rgba(26, 26, 74, 0.6);
        border: 1px solid rgba(0, 128, 255, 0.3);
        border-radius: 30px;
        color: #a8dadc;
        cursor: pointer;
        transition: all 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
        font-weight: 500;
        letter-spacing: 0.5px;
    }

    .tab-btn:hover {
        border-color: #00ffcc;
        color: #fff;
        box-shadow: 0 0 15px rgba(0, 255, 204, 0.3);
        transform: translateY(-2px);
    }

    .tab-btn.active {
        background: linear-gradient(
            135deg,
            rgba(0, 255, 204, 0.2) 0%,
            rgba(0, 128, 255, 0.2) 100%
        );
        border-color: #00ffcc;
        color: #00ffcc;
        box-shadow: 0 0 20px rgba(0, 255, 204, 0.4);
    }

    .content-viewport {
        flex: 1;
        position: relative;
        overflow: hidden;
        padding: 1rem 2rem;
    }

    .view-pane {
        width: 100%;
        height: 100%;
        position: relative;
    }

    @media (max-width: 768px) {
        .dashboard-grid {
            grid-template-columns: 1fr;
        }
        .sidebar {
            position: absolute;
            z-index: 50;
            width: 100%;
            height: 100%;
        }
    }
</style>
