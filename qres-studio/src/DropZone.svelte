<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { listen } from "@tauri-apps/api/event";
    import { save, open } from "@tauri-apps/plugin-dialog";
    import { createEventDispatcher } from "svelte";
    import { toast } from "@zerodevx/svelte-toast";
    import ArchiveView from "./ArchiveView.svelte";

    const dispatch = createEventDispatcher();
    let isDragging = false;
    let isProcessing = false;
    let progress = 0;
    let currentRatio = 0;
    let activeEngine = "zstd";
    let currentFile = "";
    let isTrainable = false;

    // New State for Archive Browsing
    let isViewingArchive = false;
    let archiveManifest: any = null;
    let currentArchivePath = "";

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

        // Try standard files list first (Tauri often populates .path here)
        if (e.dataTransfer?.files && e.dataTransfer.files.length > 0) {
            const file = e.dataTransfer.files[0];
            // @ts-ignore - Tauri adds 'path' to File object
            const path = file.path || file.name;

            // Simple check (refine if needed using FileSystemHandle)
            await processFile(path);
            return;
        }

        const items = e.dataTransfer?.items;
        if (!items || items.length === 0) return;
        const item = items[0];
        if (item.kind === "file") {
            // @ts-ignore
            const entry = await (item as any).getAsFileSystemHandle();
            if (!entry) return;
            // @ts-ignore
            if (entry.kind === "directory") {
                await processFolder(entry);
            } else {
                // @ts-ignore
                const file = await entry.getFile();
                // Prefer file.path (absolute) if available, else name
                await processFile(file.path || file.name);
            }
        }
    }

    async function processFile(filePath: string) {
        // @ts-ignore
        if (!window.__TAURI__) {
            toast.push('File processing not available in browser mode');
            return;
        }
        // Detect if it is a QRES archive (check extension)
        const lower = filePath.toLowerCase();
        const isArchive = lower.endsWith(".qres") || lower.endsWith(".qrar");

        if (isArchive) {
            try {
                // STEP 1: Try to Browse as Archive (QRAR)
                // This works for new solid archives
                const manifest = await invoke("browse_archive", {
                    archivePath: filePath,
                });

                // Show UI
                archiveManifest = manifest;
                currentArchivePath = filePath;
                isViewingArchive = true;
            } catch (e) {
                // Fallback: If browse fails, it's likely a legacy/single-file .qres (Stream format)
                // or a corrupt file. We treat it as a single file stream.
                console.warn(
                    "Archive browse failed, attempting single-stream decompression",
                    e,
                );
                toast.push(`Archive browse failed: ${e}`);
                await startSingleFileDecompression(filePath);
            }
        } else {
            // Normal Compression Logic (e.g. dropping a PDF, Image, Doc)
            await startCompression(filePath);
        }
    }

    // New method for single file stream decompression (Legacy .qres)
    async function startSingleFileDecompression(filePath: string) {
        try {
            const fileName =
                filePath.split("\\").pop() || filePath.split("/").pop();
            // Remove .qres extension if present to suggest original name
            const defaultName = fileName
                ? fileName.replace(/\.qres$/i, "")
                : "extracted";

            const destPath = await save({
                defaultPath: defaultName,
                title: "Save decompressed file",
            });
            if (!destPath) return;

            isProcessing = true;
            currentFile = "Decompressing Stream...";

            const unlisten = await listen<{
                percent: number;
                status: string;
            }>("decompression-progress", (event) => {
                progress = event.payload.percent;
            });

            // Use decompress_file (Stream logic)
            await invoke("decompress_file", {
                src: filePath,
                dest: destPath,
            });

            unlisten();
            isProcessing = false;
            progress = 0;
            dispatch("complete");
        } catch (error) {
            console.error("Decompression error:", error);
            isProcessing = false;
            alert("Error: " + error);
        }
    }

    async function startDecompression(filePath: string) {
        // This is now triggered by the ArchiveView, so it's strictly for extracting the full archive
        try {
            const destPath = await open({
                directory: true,
                title: "Select extraction folder",
            });
            if (!destPath) return;

            isProcessing = true;
            currentFile = "Extracting Archive...";

            const unlisten = await listen<{
                percent: number;
                status: string;
            }>("extraction-progress", (event) => {
                progress = event.payload.percent;
                currentFile = event.payload.status;
            });

            await invoke("extract_archive", {
                archivePath: filePath,
                outputDir: destPath,
            });

            unlisten();
            isProcessing = false;
            progress = 0;
            isViewingArchive = false;
            dispatch("complete");
        } catch (error) {
            console.error("Extraction error:", error);
            isProcessing = false;
            alert("Error: " + error);
        }
    }

    async function startCompression(filePath: string) {
        try {
            const fileName =
                filePath.split("\\").pop() || filePath.split("/").pop();
            // Create a .qrar archive by default for all new conversions
            const destPath = await save({
                defaultPath: (fileName || "archive") + ".qrar",
                title: "Save archive",
            });
            if (!destPath) return;

            isProcessing = true;
            currentFile = fileName || "";

            const unlisten = await listen<CompressionProgressPayload>(
                "compression-progress",
                (event) => {
                    progress = event.payload.percent;
                    if (event.payload.current_ratio)
                        currentRatio = event.payload.current_ratio;
                    if (event.payload.active_engine)
                        activeEngine = event.payload.active_engine;
                    if (event.payload.file) currentFile = event.payload.file;
                },
            );

            // Use compress_file which points to commands::compress_file
            // That command handles both generic file compression and folder compression
            // But we should send "folder" logic if it's a folder?
            // commands::compress_file already checks if src_path.is_dir()! So we can just call it.

            const result = (await invoke("compress_file", {
                src: filePath,
                dest: destPath,
            })) as any;

            unlisten();
            isProcessing = false;
            progress = 0;

            // Check if trainable
            if (result && result.is_trainable) {
                isTrainable = true;
                if (
                    confirm(
                        "This is a data file! Would you like to train the meta-brain on it?",
                    )
                ) {
                    await trainOnFile(filePath);
                }
            }
            isTrainable = false;
        } catch (error) {
            alert("Error: " + error);
            isProcessing = false;
        }
    }

    async function processFolder(dirHandle: any) {
        // Just use startCompression
        // But we need the path.
        // If we came from processFolder(entry) via FileSystemHandle, we might not have full path easily.
        // However, if we came from e.dataTransfer.files, we have it.
        // For now, let's just trigger the 'Compress Folder' logic if we can get a path.
        // The previous code had a placeholder alert.
        // Let's try to get path from the handle if possible, otherwise rely on the Dialog approach.
        alert(
            "Folder drag detected. Please use the 'Compress Folder' menu option for now to ensure proper path resolution.",
        );
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
    {#if isViewingArchive}
        <ArchiveView
            manifest={archiveManifest}
            archivePath={currentArchivePath}
            on:close={() => (isViewingArchive = false)}
            on:extractAll={() => startDecompression(currentArchivePath)}
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
                            {activeEngine
                                ? activeEngine.toUpperCase()
                                : "PROCESSING"}
                        </div>
                        {#if currentRatio}
                            <div class="ratio">
                                Ratio: {currentRatio.toFixed(2)}
                            </div>
                        {/if}
                        <div class="file-name">{currentFile}</div>
                    </div>
                {:else}
                    <div class="drop-prompt">
                        <div class="icon">📦</div>
                        <p>Drop file or folder here</p>
                        <small
                            >Compress PDFs, Images, or extract .qres/.qrar</small
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
        position: relative; /* For absolute positioning of ArchiveView */
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
