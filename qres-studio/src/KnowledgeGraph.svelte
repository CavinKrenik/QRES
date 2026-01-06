<script>
    // @ts-nocheck
    import { onMount } from "svelte";
    import * as d3 from "d3";

    /** @type {any} */
    let svg;
    let width = 800;
    let height = 600;

    // Stable Sample Data
    let nodes = [
        { id: "Root", group: 1 },
        { id: "SNN", group: 2 },
        { id: "QNN", group: 2 },
        { id: "Mixer", group: 3 },
        { id: "Swarm", group: 3 },
    ];
    let links = [
        { source: "Root", target: "SNN" },
        { source: "Root", target: "QNN" },
        { source: "SNN", target: "Mixer" },
        { source: "QNN", target: "Mixer" },
        { source: "Mixer", target: "Swarm" },
    ];

    onMount(() => {
        if (!svg) return;

        const rect = svg.getBoundingClientRect();
        width = rect.width || 800;
        height = rect.height || 600;

        const simulation = d3
            .forceSimulation(nodes)
            .force(
                "link",
                d3
                    .forceLink(links)
                    .id((d) => d.id)
                    .distance(100),
            )
            .force("charge", d3.forceManyBody().strength(-300))
            .force("center", d3.forceCenter(width / 2, height / 2))
            .force("collide", d3.forceCollide(30));

        const svgEl = d3.select(svg);
        svgEl.selectAll("*").remove();

        const g = svgEl.append("g");
        svgEl.call(
            d3
                .zoom()
                .extent([
                    [0, 0],
                    [width, height],
                ])
                .scaleExtent([0.1, 4])
                .on("zoom", (event) => {
                    g.attr("transform", event.transform);
                }),
        );

        const link = g
            .append("g")
            .attr("stroke", "#999")
            .attr("stroke-opacity", 0.6)
            .selectAll("line")
            .data(links)
            .join("line")
            .attr("stroke-width", 2);

        const node = g
            .append("g")
            .attr("stroke", "#fff")
            .attr("stroke-width", 1.5)
            .selectAll("circle")
            .data(nodes)
            .join("circle")
            .attr("r", 10)
            .attr("fill", (d) => (d.group === 1 ? "#e94560" : "#1a5490"))
            .call(drag(simulation));

        const labels = g
            .append("g")
            .selectAll("text")
            .data(nodes)
            .join("text")
            .text((d) => d.id)
            .attr("x", 12)
            .attr("y", 3)
            .style("fill", "#a8dadc")
            .style("font-size", "12px")
            .style("pointer-events", "none");

        simulation.on("tick", () => {
            link.attr("x1", (d) => d.source.x)
                .attr("y1", (d) => d.source.y)
                .attr("x2", (d) => d.target.x)
                .attr("y2", (d) => d.target.y);

            node.attr("cx", (d) => d.x).attr("cy", (d) => d.y);

            labels.attr("x", (d) => d.x + 12).attr("y", (d) => d.y + 3);
        });
    });

    function drag(simulation) {
        function dragstarted(event) {
            if (!event.active) simulation.alphaTarget(0.3).restart();
            event.subject.fx = event.subject.x;
            event.subject.fy = event.subject.y;
        }

        function dragged(event) {
            event.subject.fx = event.subject.x;
            event.subject.fy = event.subject.y;
        }

        function dragended(event) {
            if (!event.active) simulation.alphaTarget(0);
            event.subject.fx = null;
            event.subject.fy = null;
        }

        return d3
            .drag()
            .on("start", dragstarted)
            .on("drag", dragged)
            .on("end", dragended);
    }
</script>

<div class="graph-container">
    <svg bind:this={svg} width="100%" height="100%"></svg>
</div>

<style>
    .graph-container {
        width: 100%;
        height: 100%;
        background: #0a0a2a;
        border-radius: 8px;
        overflow: hidden;
        min-height: 400px;
    }

    svg {
        display: block;
        cursor: grab;
    }

    svg:active {
        cursor: grabbing;
    }
</style>
