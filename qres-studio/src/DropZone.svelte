<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import { toast } from "@zerodevx/svelte-toast";
    import { engine } from "./lib/compressionEngine"; // Real Hybrid Engine
    import ArchiveView from "./ArchiveView.svelte";

    const dispatch = createEventDispatcher();
    let isDragging = false;
    let isProcessing = false;
    let progress = 0;
    let activeEngine = "HYBRID";
    let currentFile = "";

    // Archive State
    let isViewingArchive = false;
    let archiveManifest: any = null;
    let currentArchivePath = "";

    const engineColors: Record<string, string> = {
        zstd: "#fbbf24", // Gold
        linear: "#3b82f6", // Blue
        ipeps: "#10b981", // Green
        lstm: "#a855f7", // Purple
        HYBRID: "#e94560", // QRES Red
    };

    let isNative = false;

    // Check environment safely
    if (typeof window !== "undefined") {
        // @ts-ignore
        isNative = !!window.__TAURI__;
    }

    async function handleDrop(e: DragEvent) {
        e.preventDefault();
        isDragging = false;

        if (e.dataTransfer?.files && e.dataTransfer.files.length > 0) {
            const file = e.dataTransfer.files[0];
            await processFile(file);
        }
    }

    async function processFile(file: File) {
        isProcessing = true;
        currentFile = file.name;
        progress = 10;
        activeEngine = "HYBRID";

        try {
            const useWasm = !isNative;

            console.log(
                `🚀 Processing ${file.name} (Engine: ${useWasm ? "WASM" : "Native"})`,
            );

            const buffer = await file.arrayBuffer();
            const bytes = new Uint8Array(buffer);
            progress = 40;

            const result = await engine.compress(bytes, useWasm);

            progress = 100;
            const ratio = (result.ratio * 100).toFixed(2);
            toast.push(
                `✅ Success! Ratio: ${ratio}% (${result.duration_ms.toFixed(0)}ms)`,
            );

            dispatch("complete", result);
        } catch (error) {
            console.error("Compression Error:", error);
            toast.push(`❌ Error: ${error}`);
        } finally {
            setTimeout(() => {
                isProcessing = false;
                progress = 0;
            }, 1500);
        }
    }

    function handleDragOver(e: DragEvent) {
        e.preventDefault();
        isDragging = true;
    }

    function handleDragLeave() {
        isDragging = false;
    }
</script>

<div class="drop-zone-container">
    {#if isViewingArchive}
        <ArchiveView
            manifest={archiveManifest}
            archivePath={currentArchivePath}
            on:close={() => (isViewingArchive = false)}
        />
    {:else}
        <div
            class="drop-zone"
            class:dragging={isDragging}
            class:processing={isProcessing}
            role="button"
            tabindex="0"
            on:drop={handleDrop}
            on:dragover={handleDragOver}
            on:dragleave={handleDragLeave}
        >
            <svg class="ring" viewBox="0 0 200 200">
                <circle
                    cx="100"
                    cy="100"
                    r="80"
                    fill="none"
                    stroke={engineColors[activeEngine] || "#ccc"}
                    stroke-width="4"
                    stroke-dasharray="502"
                    stroke-dashoffset={502 - (502 * progress) / 100}
                    class:pulsing={isProcessing}
                />
            </svg>

            <div class="drop-content">
                {#if isProcessing}
                    <div class="progress-info">
                        <div class="percent">{progress.toFixed(0)}%</div>
                        <div
                            class="engine"
                            style="color: {engineColors[activeEngine]}"
                        >
                            {activeEngine}
                        </div>
                        <div class="file-name">{currentFile}</div>
                    </div>
                {:else}
                    <div class="drop-prompt">
                        <div class="icon">📦</div>
                        <p>Drop file to Compress</p>
                        <!-- @ts-ignore -->
                        <small
                            >{isNative
                                ? "Native Mode Ready"
                                : "WASM Browser Mode Ready"}</small
                        >
                    </div>
                {/if}
            </div>
        </div>
    {/if}
</div>

<style>
    .drop-zone-container {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        height: 100%;
        padding: 2rem;
        position: relative;
    }

    .drop-zone {
        position: relative;
        width: min(400px, 80vw);
        height: min(400px, 60vh);
        aspect-ratio: 1;
        display: flex;
        align-items: center;
        justify-content: center;
        border: 2px dashed rgba(99, 102, 241, 0.3);
        border-radius: 50%;
        transition: all 0.3s;
        cursor: pointer;
    }

    .drop-zone.dragging {
        border-color: #818cf8;
        background: rgba(99, 102, 241, 0.1);
        transform: scale(1.05);
    }

    .ring {
        position: absolute;
        width: 100%;
        height: 100%;
        transform: rotate(-90deg);
        filter: drop-shadow(0 0 20px currentColor);
    }

    .ring circle {
        transition:
            stroke-dashoffset 0.3s,
            stroke 0.5s;
    }

    .ring circle.pulsing {
        animation: pulse 2s infinite;
    }

    @keyframes pulse {
        0%,
        100% {
            opacity: 1;
        }
        50% {
            opacity: 0.6;
        }
    }

    .drop-content {
        position: relative;
        z-index: 1;
        text-align: center;
    }

    .drop-prompt .icon {
        font-size: 4rem;
        margin-bottom: 1rem;
    }

    .drop-prompt p {
        margin: 0;
        font-size: 1.5rem;
        font-weight: 600;
    }

    .drop-prompt small {
        color: #94a3b8;
        font-size: 0.9rem;
    }

    .progress-info {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .percent {
        font-size: 3rem;
        font-weight: 700;
    }

    .engine {
        font-size: 1.2rem;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 2px;
    }

    .file-name {
        font-size: 0.75rem;
        color: #64748b;
        margin-top: 0.5rem;
        max-width: 200px;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }
</style>
