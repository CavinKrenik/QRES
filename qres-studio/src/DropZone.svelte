<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { listen } from "@tauri-apps/api/event";
    import { save, open } from "@tauri-apps/plugin-dialog";
    import { createEventDispatcher } from "svelte";

    const dispatch = createEventDispatcher();

    let isDragging = false;
    let isProcessing = false;
    let progress = 0;
    let currentRatio = 0;
    let activeEngine = "zstd";
    let currentFile = "";
    let isTrainable = false;

    const engineColors: Record<string, string> = {
        zstd: "#fbbf24", // Gold
        linear: "#3b82f6", // Blue
        ipeps: "#10b981", // Green
        lstm: "#a855f7", // Purple
    };

    interface CompressionProgressPayload {
        percent: number;
        current_ratio: number;
        active_engine: string;
        file?: string;
    }

    async function handleDrop(e: DragEvent) {
        e.preventDefault();
        isDragging = false;

        const items = e.dataTransfer?.items;
        if (!items || items.length === 0) return;

        // Handle first item (file or folder)
        const item = items[0];

        if (item.kind === "file") {
            const entry = await item.getAsFileSystemHandle();
            if (!entry) return;

            // @ts-ignore - FileSystemHandle types
            if (entry.kind === "directory") {
                await processFolder(entry);
            } else {
                // @ts-ignore
                const file = await entry.getFile();
                await processFile(file.path || file.name);
            }
        }
    }

    async function processFile(filePath: string) {
        const isQresFile = filePath.endsWith(".qres");

        try {
            let destPath: string | null;

            if (isQresFile) {
                // Decompress mode
                destPath = await save({
                    defaultPath: filePath.replace(".qres", ""),
                    title: "Save decompressed file",
                });
                if (!destPath) return;

                isProcessing = true;
                currentFile =
                    filePath.split("\\").pop() ||
                    filePath.split("/").pop() ||
                    "";

                const unlisten = await listen<{
                    percent: number;
                    status: string;
                }>("decompression-progress", (event) => {
                    progress = event.payload.percent;
                });

                await invoke("decompress_file", {
                    src: filePath,
                    dest: destPath,
                });
                unlisten();
            } else {
                // Compress mode
                const fileName =
                    filePath.split("\\").pop() || filePath.split("/").pop();
                destPath = await save({
                    defaultPath: (fileName || "archive") + ".qres",
                    title: "Save compressed file",
                });
                if (!destPath) return;

                isProcessing = true;
                currentFile = fileName || "";

                const unlisten = await listen<CompressionProgressPayload>(
                    "compression-progress",
                    (event) => {
                        progress = event.payload.percent;
                        currentRatio = event.payload.current_ratio;
                        activeEngine = event.payload.active_engine;
                        if (event.payload.file) {
                            currentFile = event.payload.file;
                        }
                    },
                );

                const result = (await invoke("compress_file", {
                    src: filePath,
                    dest: destPath,
                })) as any;

                unlisten();

                // Check if trainable
                if (result.is_trainable) {
                    isTrainable = true;
                    if (
                        confirm(
                            "This is a data file! Would you like to train the meta-brain on it?",
                        )
                    ) {
                        await trainOnFile(filePath);
                    }
                }
            }

            isProcessing = false;
            progress = 0;
            isTrainable = false;
            dispatch("complete");
        } catch (error) {
            console.error("Error processing file:", error);
            isProcessing = false;
            alert("Error: " + error);
        }
    }

    async function processFolder(dirHandle: any) {
        try {
            const destPath = await open({
                directory: true,
                title: "Select destination folder for compressed files",
            });

            if (!destPath) return;

            isProcessing = true;
            currentFile = "Processing folder...";

            const unlisten = await listen<CompressionProgressPayload>(
                "compression-progress",
                (event) => {
                    progress = event.payload.percent;
                    currentRatio = event.payload.current_ratio;
                    activeEngine = event.payload.active_engine;
                    if (event.payload.file) {
                        currentFile = event.payload.file;
                    }
                },
            );

            // Get folder path from handle
            // @ts-ignore
            const folderPath = dirHandle.name; // This is simplified - actual implementation would need proper path resolution

            await invoke("compress_file", {
                src: folderPath,
                dest: destPath,
            });

            unlisten();
            isProcessing = false;
            progress = 0;
            dispatch("complete");
        } catch (error) {
            console.error("Error processing folder:", error);
            isProcessing = false;
            alert("Error: " + error);
        }
    }

    async function trainOnFile(filePath: string) {
        try {
            const result = (await invoke("train_on_file", {
                filePath,
            })) as string;
            alert("Training complete!\n" + result);
        } catch (error) {
            alert("Training failed: " + error);
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
                        {activeEngine ? activeEngine.toUpperCase() : "INIT"}
                    </div>
                    <div class="ratio">
                        Ratio: {currentRatio ? currentRatio.toFixed(2) : "..."}
                    </div>
                    {#if currentFile}
                        <div class="file-name">{currentFile}</div>
                    {/if}
                </div>
            {:else}
                <div class="drop-prompt">
                    <div class="icon">📦</div>
                    <p>Drop file or folder here</p>
                    <small>compress any file/folder</small>
                </div>
            {/if}
        </div>
    </div>
</div>

<style>
    .drop-zone-container {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        height: 100%;
        padding: 2rem;
        gap: 2rem;
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

    .ratio {
        font-size: 0.9rem;
        color: #94a3b8;
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
