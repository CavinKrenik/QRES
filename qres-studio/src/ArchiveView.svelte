<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import { invoke } from "@tauri-apps/api/core";

    export let manifest: any;
    // Archive path for future use (external reference)
    export const archivePath: string = "";

    const dispatch = createEventDispatcher();
    let selectedFiles: Set<string> = new Set();
    let isExtracting = false;

    function formatBytes(bytes: number) {
        if (bytes === 0) return "0 B";
        const k = 1024;
        const sizes = ["B", "KB", "MB", "GB", "TB"];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
    }

    function toggleFile(path: string) {
        if (selectedFiles.has(path)) {
            selectedFiles.delete(path);
        } else {
            selectedFiles.add(path);
        }
        selectedFiles = selectedFiles; // Trigger reactivity
    }

    async function extractSelected() {
        // Implementation for partial extraction would go here
        // For now, let's just trigger full extraction via parent
        dispatch("extractAll");
    }
</script>

<div class="archive-view">
    <div class="header">
        <h2>Archive Content</h2>
        <div class="stats">
            <span>Total Size: {formatBytes(manifest.total_size)}</span>
            <span>Files: {manifest.file_count}</span>
            <span class="method">{manifest.compression_method}</span>
        </div>
    </div>

    <div class="file-list">
        <table>
            <thead>
                <tr>
                    <th><input type="checkbox" disabled /></th>
                    <th>Name</th>
                    <th>Size</th>
                    <th>Hash</th>
                </tr>
            </thead>
            <tbody>
                {#each manifest.files as file}
                    <tr
                        class:selected={selectedFiles.has(file.path)}
                        on:click={() => toggleFile(file.path)}
                    >
                        <td>
                            <input
                                type="checkbox"
                                checked={selectedFiles.has(file.path)}
                            />
                        </td>
                        <td class="path">{file.path}</td>
                        <td>{formatBytes(file.original_size || file.size)}</td>
                        <td class="hash"
                            >{file.hash
                                ? file.hash.substring(0, 8) + "..."
                                : "-"}</td
                        >
                    </tr>
                {/each}
            </tbody>
        </table>
    </div>

    <div class="actions">
        <button class="secondary" on:click={() => dispatch("close")}
            >Close</button
        >
        <button class="primary" on:click={() => dispatch("extractAll")}>
            Extract All
        </button>
    </div>
</div>

<style>
    .archive-view {
        position: absolute;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background: #1e1e2e;
        z-index: 10;
        display: flex;
        flex-direction: column;
        padding: 1rem;
        box-sizing: border-box;
    }

    .header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 1rem;
        border-bottom: 1px solid #313244;
        padding-bottom: 1rem;
    }

    .stats {
        display: flex;
        gap: 1rem;
        font-size: 0.9rem;
        color: #a6adc8;
    }

    .method {
        background: #313244;
        padding: 0.2rem 0.5rem;
        border-radius: 4px;
        color: #89b4fa;
    }

    .file-list {
        flex: 1;
        overflow-y: auto;
        background: #181825;
        border-radius: 8px;
    }

    table {
        width: 100%;
        border-collapse: collapse;
        color: #cdd6f4;
    }

    th {
        text-align: left;
        padding: 0.75rem;
        background: #313244;
        position: sticky;
        top: 0;
    }

    td {
        padding: 0.75rem;
        border-bottom: 1px solid #313244;
    }

    tr:hover {
        background: #313244;
        cursor: pointer;
    }

    tr.selected {
        background: rgba(137, 180, 250, 0.2);
    }

    .path {
        font-family: monospace;
    }

    .hash {
        font-family: monospace;
        color: #6c7086;
        font-size: 0.8rem;
    }

    .actions {
        display: flex;
        justify-content: flex-end;
        gap: 1rem;
        padding-top: 1rem;
    }

    button {
        padding: 0.5rem 1.5rem;
        border-radius: 6px;
        border: none;
        cursor: pointer;
        font-weight: 600;
        transition: transform 0.1s;
    }

    button:active {
        transform: scale(0.98);
    }

    button.primary {
        background: #89b4fa;
        color: #1e1e2e;
    }

    button.secondary {
        background: #313244;
        color: #cdd6f4;
    }
</style>
