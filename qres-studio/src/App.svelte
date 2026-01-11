<script lang="ts">
    import { SvelteToast } from "@zerodevx/svelte-toast";
    import { fade } from "svelte/transition";
    import StarshipHeader from "./components/StarshipHeader.svelte";
    import StarshipSidebar from "./components/StarshipSidebar.svelte";
    import KnowledgeGraph from "./KnowledgeGraph.svelte";
    import HiveMind from "./HiveMind.svelte";
    import IoTDashboard from "./components/IoTDashboard.svelte";

    let currentTab = "iot"; // Default to IoT Stream

    function handleNavigate(event: CustomEvent<string>) {
        currentTab = event.detail;
    }
</script>

<SvelteToast />

<div class="app-shell">
    <StarshipHeader />

    <main class="main-layout">
        <StarshipSidebar activeTab={currentTab} on:navigate={handleNavigate} />

        <section class="viewport">
            {#if currentTab === "iot"}
                <div class="view-container" transition:fade={{ duration: 150 }}>
                    <IoTDashboard />
                </div>
            {:else if currentTab === "hive"}
                <div class="view-container" transition:fade={{ duration: 150 }}>
                    <HiveMind />
                </div>
            {:else if currentTab === "graph"}
                <div class="view-container" transition:fade={{ duration: 150 }}>
                    <KnowledgeGraph />
                </div>
            {/if}
        </section>
    </main>
</div>

<style>
    :global(*) {
        box-sizing: border-box;
    }

    :global(html, body) {
        margin: 0;
        padding: 0;
        height: 100%;
        font-family:
            "Inter",
            -apple-system,
            BlinkMacSystemFont,
            "Segoe UI",
            sans-serif;
        overflow: hidden;
        background: #050510;
    }

    .app-shell {
        display: flex;
        flex-direction: column;
        height: 100vh;
        width: 100vw;
        color: #ffffff;
        background: #050510;
    }

    .main-layout {
        display: flex;
        flex: 1;
        gap: 0.5rem;
        padding: 0.5rem;
        overflow: hidden;
        min-height: 0; /* Important for flex children to shrink */
    }

    .viewport {
        flex: 1;
        position: relative;
        overflow: hidden;
        border-radius: 8px;
        background: #0a0a1a;
        border: 1px solid rgba(0, 255, 204, 0.15);
        min-width: 0; /* Important for flex children */
    }

    .view-container {
        height: 100%;
        width: 100%;
        overflow: hidden;
    }

    @media (max-width: 768px) {
        .main-layout {
            flex-direction: column;
        }
    }
</style>
