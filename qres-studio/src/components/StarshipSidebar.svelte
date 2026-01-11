<script lang="ts">
    import { createEventDispatcher } from "svelte";

    export let activeTab: string = "iot";

    const dispatch = createEventDispatcher();

    function navigate(tab: string) {
        activeTab = tab;
        dispatch("navigate", tab);
    }

    const navItems = [
        { id: "iot", icon: "[~]", label: "IoT Stream" },
        { id: "hive", icon: "[*]", label: "Network Map" },
        { id: "graph", icon: "[+]", label: "Neural Graph" },
    ];
</script>

<aside class="sidebar">
    <div class="nav-header">
        <span class="nav-title">Navigation</span>
    </div>

    <nav class="nav-list">
        {#each navItems as item}
            <button
                class="nav-item"
                class:active={activeTab === item.id}
                on:click={() => navigate(item.id)}
            >
                <span class="nav-icon">{item.icon}</span>
                <span class="nav-label">{item.label}</span>
            </button>
        {/each}
    </nav>

    <div class="sidebar-footer">
        <div class="version-badge">v15.3.0</div>
    </div>
</aside>

<style>
    .sidebar {
        display: flex;
        flex-direction: column;
        height: 100%;
        background: rgba(5, 5, 16, 0.95);
        border: 1px solid rgba(0, 255, 204, 0.2);
        border-radius: 8px;
    }

    .nav-header {
        padding: 1rem;
        border-bottom: 1px solid rgba(0, 255, 204, 0.1);
    }

    .nav-title {
        font-size: 0.7rem;
        color: #666;
        text-transform: uppercase;
        letter-spacing: 2px;
        font-family: "JetBrains Mono", monospace;
    }

    .nav-list {
        flex: 1;
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
        padding: 0.5rem;
    }

    .nav-item {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        padding: 0.8rem 1rem;
        background: transparent;
        border: 1px solid transparent;
        border-radius: 6px;
        color: #888;
        cursor: pointer;
        transition: all 0.2s ease;
        text-align: left;
        font-size: 0.9rem;
    }

    .nav-item:hover {
        background: rgba(0, 255, 204, 0.05);
        border-color: rgba(0, 255, 204, 0.2);
        color: #fff;
    }

    .nav-item.active {
        background: rgba(0, 255, 204, 0.1);
        border-color: #00ffcc;
        color: #00ffcc;
    }

    .nav-icon {
        font-size: 1.2rem;
    }

    .nav-label {
        font-family: "JetBrains Mono", monospace;
        font-weight: 500;
    }

    .sidebar-footer {
        padding: 1rem;
        border-top: 1px solid rgba(0, 255, 204, 0.1);
        display: flex;
        justify-content: center;
    }

    .version-badge {
        padding: 0.3rem 0.8rem;
        background: rgba(255, 68, 68, 0.1);
        border: 1px solid rgba(255, 68, 68, 0.3);
        border-radius: 4px;
        color: #ff4444;
        font-family: "JetBrains Mono", monospace;
        font-size: 0.7rem;
        letter-spacing: 1px;
    }
</style>
