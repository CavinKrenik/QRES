<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { bandwidthHistory } from "../lib/iotStore";
    import * as d3 from "d3";

    let svgElement: SVGSVGElement;
    let container: HTMLDivElement;
    let width = 0;
    let height = 0;

    // Chart config
    const margin = { top: 20, right: 20, bottom: 30, left: 50 };
    const maxPoints = 50;

    // D3 Scales
    let x: d3.ScaleTime<number, number>;
    let y: d3.ScaleLinear<number, number>;
    let lineRaw: d3.Line<any>;
    let lineComp: d3.Line<any>;

    $: data = $bandwidthHistory;

    function updateChart() {
        if (!svgElement || !data || data.length < 2) return;

        // Update domains
        const now = Date.now();
        x.domain([now - 10000, now]); // 10 second rolling window

        // Auto-scale Y with a minimum of 100 bytes
        const maxY = Math.max(100, d3.max(data, (d) => d.rawBytes) || 100);
        y.domain([0, maxY * 1.2]);

        const svg = d3.select(svgElement);

        // Update axes
        svg.select<SVGGElement>(".x-axis").call(d3.axisBottom(x).ticks(5));
        svg.select<SVGGElement>(".y-axis").call(d3.axisLeft(y).ticks(5));

        // Update lines
        svg.select(".line-raw").datum(data).attr("d", lineRaw);

        svg.select(".line-comp").datum(data).attr("d", lineComp);
    }

    function initChart() {
        if (!container) return;
        width = container.clientWidth - margin.left - margin.right;
        height = container.clientHeight - margin.top - margin.bottom;

        x = d3.scaleTime().range([0, width]);
        y = d3.scaleLinear().range([height, 0]);

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

        const svg = d3
            .select(svgElement)
            .attr("width", width + margin.left + margin.right)
            .attr("height", height + margin.top + margin.bottom)
            .append("g")
            .attr("transform", `translate(${margin.left},${margin.top})`);

        // Add axes
        svg.append("g")
            .attr("class", "x-axis")
            .attr("transform", `translate(0,${height})`);

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
    }

    onMount(() => {
        initChart();
        // Resize observer could go here
    });

    // Reactive update
    $: if (data) {
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
        min-height: 250px;
        position: relative;
        background: rgba(0, 0, 0, 0.2);
        border-radius: 8px;
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
