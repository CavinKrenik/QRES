<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { bandwidthHistory } from "../lib/iotStore";
    import * as d3 from "d3";

    let svgElement: SVGSVGElement;
    let container: HTMLDivElement;
    let width = 600;
    let height = 300;
    let resizeObserver: ResizeObserver;

    // Chart config
    const margin = { top: 20, right: 20, bottom: 30, left: 50 };

    // D3 Scales
    let x: d3.ScaleTime<number, number>;
    let y: d3.ScaleLinear<number, number>;
    let lineRaw: d3.Line<any>;
    let lineComp: d3.Line<any>;
    let initialized = false;

    $: data = $bandwidthHistory;

    function updateChart() {
        if (!svgElement || !data || data.length < 2 || !initialized) return;

        const innerWidth = width - margin.left - margin.right;
        const innerHeight = height - margin.top - margin.bottom;

        // Update scales
        const now = Date.now();
        x.domain([now - 10000, now]).range([0, innerWidth]);

        const maxY = Math.max(100, d3.max(data, (d) => d.rawBytes) || 100);
        y.domain([0, maxY * 1.2]).range([innerHeight, 0]);

        const svg = d3.select(svgElement);

        // Update axes
        svg.select<SVGGElement>(".x-axis")
            .attr("transform", `translate(0,${innerHeight})`)
            .call(d3.axisBottom(x).ticks(5));
        svg.select<SVGGElement>(".y-axis").call(d3.axisLeft(y).ticks(5));

        // Update lines
        svg.select(".line-raw").datum(data).attr("d", lineRaw);
        svg.select(".line-comp").datum(data).attr("d", lineComp);
    }

    function initChart() {
        if (!container || !svgElement) return;

        const rect = container.getBoundingClientRect();
        width = rect.width || 600;
        height = rect.height || 300;

        const innerWidth = width - margin.left - margin.right;
        const innerHeight = height - margin.top - margin.bottom;

        x = d3.scaleTime().range([0, innerWidth]);
        y = d3.scaleLinear().range([innerHeight, 0]);

        lineRaw = d3
            .line<any>()
            .x((d) => x(d.timestamp))
            .y((d) => y(d.rawBytes))
            .curve(d3.curveMonotoneX);

        lineComp = d3
            .line<any>()
            .x((d) => x(d.timestamp))
            .y((d) => y(d.compressedBytes))
            .curve(d3.curveMonotoneX);

        // Clear and recreate
        d3.select(svgElement).selectAll("*").remove();

        const svg = d3
            .select(svgElement)
            .attr("width", width)
            .attr("height", height)
            .append("g")
            .attr("transform", `translate(${margin.left},${margin.top})`);

        // Add axes
        svg.append("g")
            .attr("class", "x-axis")
            .attr("transform", `translate(0,${innerHeight})`);

        svg.append("g").attr("class", "y-axis");

        // Add paths
        svg.append("path")
            .attr("class", "line-raw")
            .attr("fill", "none")
            .attr("stroke", "#ff4444")
            .attr("stroke-width", 2);

        svg.append("path")
            .attr("class", "line-comp")
            .attr("fill", "none")
            .attr("stroke", "#00ffcc")
            .attr("stroke-width", 2);

        initialized = true;
    }

    function handleResize() {
        if (!container) return;
        const rect = container.getBoundingClientRect();
        if (rect.width > 0 && rect.height > 0) {
            width = rect.width;
            height = rect.height;
            initChart();
            updateChart();
        }
    }

    onMount(() => {
        // Wait for container to be sized
        setTimeout(() => {
            initChart();
        }, 100);

        // Watch for resize
        resizeObserver = new ResizeObserver(() => {
            handleResize();
        });
        if (container) {
            resizeObserver.observe(container);
        }
    });

    onDestroy(() => {
        if (resizeObserver) {
            resizeObserver.disconnect();
        }
    });

    // Reactive update
    $: if (data && initialized) {
        requestAnimationFrame(updateChart);
    }
</script>

<div class="chart-container" bind:this={container}>
    <svg bind:this={svgElement}></svg>
    <div class="legend">
        <span class="dot red"></span> Raw
        <span class="dot green"></span> QRES
    </div>
</div>

<style>
    .chart-container {
        width: 100%;
        height: 100%;
        flex: 1;
        position: relative;
        background: rgba(0, 0, 0, 0.2);
        border-radius: 8px;
        min-height: 200px;
    }

    svg {
        width: 100%;
        height: 100%;
        display: block;
    }

    :global(.x-axis text),
    :global(.y-axis text) {
        fill: #666;
        font-family: monospace;
    }

    :global(.domain),
    :global(.tick line) {
        stroke: #333;
    }

    .legend {
        position: absolute;
        top: 10px;
        right: 10px;
        display: flex;
        gap: 15px;
        font-size: 0.8rem;
        color: #fff;
    }

    .dot {
        width: 8px;
        height: 8px;
        border-radius: 50%;
        display: inline-block;
    }
    .red {
        background: #ff4444;
    }
    .green {
        background: #00ffcc;
    }
</style>
