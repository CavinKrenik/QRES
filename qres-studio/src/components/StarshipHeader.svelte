<script lang="ts">
    // Minimal header - just the QRES branding
</script>

<header class="neon-header">
    <h1>QRES Edge</h1>
    <div class="brand-badge">
        <span class="badge-text">QRES</span>
    </div>
</header>

<style>
    .neon-header {
        padding: 0.6rem 1.5rem;
        background: linear-gradient(
            90deg,
            #050510 0%,
            #0a0a2a 50%,
            #050510 100%
        );
        border-bottom: 1px solid rgba(0, 255, 204, 0.3);
        display: flex;
        justify-content: space-between;
        align-items: center;
        flex-shrink: 0;
    }

    h1 {
        margin: 0;
        font-size: 1.2rem;
        font-weight: 700;
        font-family: "JetBrains Mono", "Consolas", monospace;
        background: linear-gradient(135deg, #00ffcc 0%, #0080ff 100%);
        background-clip: text;
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
        letter-spacing: 2px;
    }

    .brand-badge {
        display: flex;
        align-items: center;
    }

    .badge-text {
        padding: 0.3rem 0.8rem;
        background: rgba(0, 255, 204, 0.1);
        border: 1px solid rgba(0, 255, 204, 0.4);
        border-radius: 4px;
        color: #00ffcc;
        font-family: "JetBrains Mono", monospace;
        font-size: 0.7rem;
        font-weight: 600;
        letter-spacing: 2px;
    }
</style>
