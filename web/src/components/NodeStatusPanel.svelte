<script lang="ts">
    import { nodeList, streamingActive, currentPacket } from "../lib/iotStore";

    // Auto-update status when streaming
    $: if ($streamingActive && $currentPacket) {
        // Update first node to match simulated data status
        // Map 'IDLE' to 'OFFLINE' for NodeStatus compatibility
        const mappedStatus =
            $currentPacket.status === "IDLE"
                ? "OFFLINE"
                : $currentPacket.status;
        $nodeList[0].status = mappedStatus;
        $nodeList[0].lastSeen = Date.now();

        // Randomly flicker others
        if (Math.random() > 0.9) {
            $nodeList[1].status = "INFERRING";
            $nodeList[1].lastSeen = Date.now();
        }
    }
</script>

<div class="node-list">
    {#each $nodeList as node}
        <div class="node-card" class:active={$streamingActive}>
            <div class="node-icon">
                <div
                    class="status-dot"
                    class:online={node.status !== "OFFLINE"}
                ></div>
            </div>
            <div class="node-info">
                <span class="name">{node.name}</span>
                <span class="type">{node.type}</span>
            </div>
            <div
                class="node-status"
                class:inferring={node.status === "INFERRING"}
                class:learning={node.status === "LEARNING"}
            >
                {node.status}
            </div>
        </div>
    {/each}
</div>

<style>
    .node-list {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .node-card {
        display: flex;
        align-items: center;
        background: rgba(255, 255, 255, 0.05);
        padding: 0.8rem;
        border-radius: 4px;
        border-left: 3px solid #333;
        transition: all 0.3s;
    }

    .node-card.active {
        border-left-color: #00ffcc;
        background: rgba(0, 255, 204, 0.05);
    }

    .node-info {
        flex-grow: 1;
        display: flex;
        flex-direction: column;
        margin-left: 10px;
    }

    .name {
        font-weight: bold;
        font-size: 0.9rem;
        color: #fff;
    }

    .type {
        font-size: 0.7rem;
        color: #888;
    }

    .status-dot {
        width: 8px;
        height: 8px;
        background: #555;
        border-radius: 50%;
    }

    .status-dot.online {
        background: #00ffcc;
        box-shadow: 0 0 5px #00ffcc;
    }

    .node-status {
        font-size: 0.7rem;
        padding: 2px 6px;
        border-radius: 3px;
        background: #222;
        color: #666;
    }

    .node-status.inferring {
        background: rgba(0, 255, 204, 0.2);
        color: #00ffcc;
    }

    .node-status.learning {
        background: rgba(255, 200, 0, 0.2);
        color: #ffcc00;
        animation: pulse-yellow 1s infinite;
    }

    @keyframes pulse-yellow {
        0% {
            opacity: 0.6;
        }
        100% {
            opacity: 1;
        }
    }
</style>
