# QRES Studio v8.2 - Starship GUI Revamp (UPDATED)
**Branch**: `feature/gui-revamp`  
**Goal**: Transform QRES Studio into a futuristic, intuitive Starship-style dashboard  
**Updated**: January 2, 2026 with enhanced Starship-specific features

---

## Design Philosophy

**Inspiration**: SpaceX Starship control interfaces (Figma recreations, Dribbble HUDs)
- **Futuristic**: Neon glows, holographic overlays, particle backgrounds, radial gauges
- **Simple**: Minimal UI, auto-updates, zero ambiguity, grid-based layout
- **Intuitive**: Hover glows, tooltips on everything, instant feedback via toasts
- **Beautiful**: Dark space theme, responsive grids, smooth 60fps animations
- **NEW**: Error handling with toasts, symmetric compress/decompress, file previews

---

## Implementation Plan (ENHANCED)

### Phase 1: Foundation & Error Fixes (Day 1)
- [x] Create feature branch
- [x] Install svelte-french-toast ✅
- [ ] Install chart.js and svelte-chartjs (for radial gauges)
- [ ] **FIX: Tauri invoke errors** - Wrap all JS calls in try/catch with toast feedback
- [ ] Create global layout structure with CSS Grid
- [ ] Add particle background system (stars/nebula)
- [ ] Implement base color scheme with glow shadows
- [ ] **NEW**: Add auto-save prompts after compress/decompress

### Phase 2: Core Components (Day 2)
- [ ] **Header.svelte** (version with neon pulse, real-time stats)
- [ ] **Sidebar.svelte** (controls + enhanced drop zone with file previews)
- [ ] **Dashboard.svelte** (central viz with tabs as overlays, not separate pages)
- [ ] **Toast system** for all feedback (e.g., "Compressed to /path/file.qres")
- [ ] **NEW**: Decompress button symmetric to Compress

### Phase 3: Enhanced Features & Starship Elements (Day 3)
- [ ] Real-time state management (Svelte stores for swarm/graph updates)
- [ ] Interactive graph with auto-fit (d3.zoom + viewBox for dynamic sizing)
- [ ] **Gauge clusters** for metrics (Chart.js radar charts with neon fills)
- [ ] **Radial dials** for topology (like Starship telemetry - peer count, fidelity)
- [ ] One-click compress/decompress with tauri dialog.save for outputs
- [ ] **NEW**: File thumbnails post-drop, graph updates post-compress

### Phase 4: Polish, Testing & End-to-End (Day 4)
- [ ] Animations and transitions (Svelte in:fade for tabs, glow on hover)
- [ ] Responsive design (sidebar collapses on mobile, no scrolls)
- [ ] Accessibility (ARIA labels + tooltips on all interactive elements)
- [ ] Cross-browser testing + Tauri rebuild verification
- [ ] **NEW**: End-to-end tests (compress → save prompt → graph update → swarm sync)

---

## Design Specifications (ENHANCED)

### Color Palette
```css
--bg-primary: #0a0a2a;           /* Deep space */
--bg-secondary: #1a1a4a;         /* Nebula blue */
--accent-neon: #00ffcc;          /* Cyan glow */
--accent-blue: #0080ff;          /* Electric blue */
--accent-purple: #a855f7;        /* Quantum purple */
--text-primary: #ffffff;         /* Pure white */
--text-secondary: #a8dadc;       /* Muted cyan */
--border-glow: rgba(0, 255, 204, 0.3);
--glow-shadow: 0 0 10px var(--accent-neon);  /* NEW: Holographic glow */
```

### Layout Grid (Responsive)
```
Desktop:
┌─────────────────────────────────────┐
│     Header (Stats + Neon Glow)      │
├──────────┬──────────────────────────┤
│          │                          │
│ Sidebar  │      Dashboard           │
│ (250px)  │      (Flex-1)            │
│          │                          │
│ Controls │  - Holographic Graph     │
│ + Drop   │  - Radial Gauges         │
│ Preview  │  - Real-time Feeds       │
│          │  - Swarm Topology        │
└──────────┴──────────────────────────┘

Mobile (< 768px):
┌─────────────────────────────────────┐
│            Header                   │
├─────────────────────────────────────┤
│                                     │
│         Dashboard (Full)            │
│                                     │
├─────────────────────────────────────┤
│    Sidebar (Collapsed Bottom)       │
└─────────────────────────────────────┘
```

### Key Features (ENHANCED)

**1. Holographic Background**
- Particle system with stars/nebula effect (low opacity)
- Subtle movement and depth perception
- Animated on scroll/interaction

**2. Neon Glow Effects**
- Active elements pulse with `--glow-shadow`
- Hover states trigger glow animations (CSS `animation: glow 1s infinite`)
- Progress indicators use gradient glows

**3. Auto-Feedback System**
- Toast notifications for ALL actions (compress, decompress, save, error)
- Real-time progress indicators (circular for compress, linear for download)
- Status updates without user interaction (swarm peer count, fidelity)

**4. Starship-Specific Elements**
- **Radial Gauges**: Chart.js radar charts for topology metrics (nodes, connections, fidelity)
- **Circular Dials**: Peer count, compression ratio, quantum fidelity
- **File Previews**: Thumbnails after drop (image preview, file icon for others)
- **Real-time Trajectories**: Graph edges animate on state updates

**5. Responsive Design**
- Mobile: Sidebar collapses to bottom drawer
- Tablet: Adjusted grid (sidebar 200px)
- Desktop: Full dashboard experience (sidebar 250px)
- **No scrolls**: Use `overflow: auto` only on graph container

---

## Component Architecture (DETAILED)

### App.svelte (Main Layout)
```svelte
<script>
  import Particles from './components/Particles.svelte';
  import Header from './components/Header.svelte';
  import Sidebar from './components/Sidebar.svelte';
  import Dashboard from './components/Dashboard.svelte';
  import { Toaster } from 'svelte-french-toast';
</script>

<Particles />
<Toaster />
<div class="app-grid">
  <Header />
  <Sidebar />
  <Dashboard />
</div>

<style>
  .app-grid {
    display: grid;
    grid-template-areas: "header header" "sidebar dashboard";
    grid-template-columns: 250px 1fr;
    grid-template-rows: auto 1fr;
    height: 100vh;
    background: var(--bg-primary);
  }
  @media (max-width: 768px) {
    .app-grid {
      grid-template-areas: "header" "dashboard" "sidebar";
      grid-template-columns: 1fr;
      grid-template-rows: auto 1fr auto;
    }
  }
</style>
```

### Header.svelte
- Version display with neon pulse animation
- Real-time stats (Saved MB, Files, Compression Ratio)
- Quantum fidelity indicator (circular gauge)
- **NEW**: Network status indicator (green dot if peers > 0)

### Sidebar.svelte
- **Compression Mode Selector** (Standard/Quantum with toggle)
- **Threshold Sliders** (Relevance, Noise Level)
- **Enhanced Drop Zone** (with file preview thumbnails)
- **Action Buttons**:
  - Compress (primary, neon glow)
  - Decompress (secondary, symmetric to compress)
  - Save State (if quantum mode)
  - Broadcast (if quantum mode)
- **NEW**: File preview after drop (image thumbnail or icon)

### Dashboard.svelte
- **Tabbed Sections** (as overlays, not separate pages):
  - Graph: Auto-fit D3 visualization with zoom
  - Swarm: Radial gauges for peer count, fidelity
  - Topology: Chart.js radar for network metrics
- **Auto-fit Graph**: Uses `d3.zoom()` and dynamic `viewBox`
- **Gauge Clusters**: Chart.js with neon fills and animations
- **Real-time Data Feeds**: Updates from Svelte stores

---

## Dependencies (UPDATED)

```bash
# Already installed:
npm install --legacy-peer-deps svelte-french-toast ✅

# To install:
npm install --legacy-peer-deps chart.js svelte-chartjs
npm install --legacy-peer-deps @tauri-apps/api  # For fixed invokes
```

**Note**: Using `--legacy-peer-deps` to avoid conflicts with Svelte 5.

---

## Error Fixes (CRITICAL)

### Current Issue
`TypeError: Cannot read properties of undefined (reading 'invoke')`

### Root Cause
Running in browser dev mode (`npm run dev`) instead of Tauri environment.

### Solutions
1. **For Development**: Wrap all `invoke()` calls in try/catch with fallback mock data
2. **For Production**: Run `npm run tauri dev` instead of `npm run dev`
3. **Add Error Handling**:
```javascript
import { invoke } from '@tauri-apps/api/core';
import toast from 'svelte-french-toast';

async function safeInvoke(command, args = {}) {
  try {
    return await invoke(command, args);
  } catch (error) {
    toast.error(`${command} failed: ${error}`);
    // Return mock data for dev mode
    return mockData[command] || null;
  }
}
```

---

## Success Criteria (ENHANCED)

- [ ] **Zero errors** (fixed Tauri invokes with try/catch + toasts)
- [ ] **1-2 click access** to all features + immediate visual feedback
- [ ] **Instant feedback** via toasts for compress/decompress/save/error
- [ ] **Smooth animations** (60fps, tested with Chrome DevTools)
- [ ] **Mobile responsive** (no horizontal scrolls, sidebar collapses)
- [ ] **Accessible** (WCAG AA: ARIA labels, keyboard nav, tooltips)
- [ ] **Beautiful** (wow factor on first load with particles + neon glows)
- [ ] **NEW: Starship gauges** (radial dials, real-time trajectories)
- [ ] **NEW: End-to-end flow** (drop → compress → save prompt → graph update → swarm sync)

---

## Next Steps (IMMEDIATE)

1. ✅ Install svelte-french-toast
2. Install chart.js and svelte-chartjs
3. Create Particles.svelte component
4. Create Header.svelte with neon glow
5. Create enhanced Sidebar.svelte
6. Create Dashboard.svelte with radial gauges
7. Add toast notifications to all actions
8. Test in `npm run tauri dev` environment
9. Polish animations and responsiveness
10. End-to-end testing

---

## Inspiration References

- **Figma**: Starship dashboard recreations (radial dials, real-time telemetry)
- **Dribbble**: Neon HUDs, holographic overlays
- **SpaceX**: Minimalist data-dense interfaces
- **Cyberpunk 2077**: Futuristic UI aesthetics

---

*Plan Updated: January 2, 2026*  
*Target Completion: v8.2 (3-5 days)*  
*Status: Phase 1 in progress*
