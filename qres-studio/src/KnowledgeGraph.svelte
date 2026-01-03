<script>
    import { onMount } from "svelte";
    import * as d3 from "d3";

    /** @type {SVGSVGElement} */
    let svg;
    let width = 800;
    let height = 600;

    onMount(async () => {
        // Load data
        const response = await fetch("/knowledge_graph.json");
        const data = await response.json();

        // @ts-ignore
        const nodes = data.nodes.map((d) => ({ ...d }));
        // @ts-ignore
        const links = data.edges.map((d) => ({ ...d }));

        // Simulation
        const simulation = d3
            .forceSimulation(nodes)
            .force(
                "link",
                d3
                    .forceLink(links)
                    // @ts-ignore
                    .id((d) => d.id)
                    .distance(150),
            )
            .force("charge", d3.forceManyBody().strength(-300))
            .force("center", d3.forceCenter(width / 2, height / 2));

        const svgEl = d3
            .select(svg)
            .attr("viewBox", [0, 0, width, height])
            .attr("title", "Multi-Modal Knowledge Graph");

        // Edges
        const link = svgEl
            .append("g")
            .attr("stroke", "#999")
            .attr("stroke-opacity", 0.6)
            .selectAll("line")
            .data(links)
            .join("line")
            .attr("stroke-width", (d) => Math.sqrt(d.weight * 5));

        // Nodes
        const node = svgEl
            .append("g")
            .attr("stroke", "#fff")
            .attr("stroke-width", 1.5)
            .selectAll("circle")
            .data(nodes)
            .join("circle")
            .attr("r", 10)
            .attr("fill", (d) => (d.type === "image" ? "#ff6b6b" : "#69db7c"))
            // @ts-ignore
            .call(drag(simulation));

        // Tooltips (Simple title for now)
        // @ts-ignore
        node.append("title").text((d) => d.id + ": " + (d.content || d.path));

        // Labels
        const labels = svgEl
            .append("g")
            .attr("class", "labels")
            .selectAll("text")
            .data(nodes)
            .join("text")
            .attr("dx", 12)
            .attr("dy", ".35em")
            .text((d) => d.id)
            .style("fill", "#ccc")
            .style("font-size", "12px")
            .style("font-family", "monospace");

        simulation.on("tick", () => {
            link.attr("x1", (d) => d.source.x)
                .attr("y1", (d) => d.source.y)
                .attr("x2", (d) => d.target.x)
                .attr("y2", (d) => d.target.y);

            node.attr("cx", (d) => d.x).attr("cy", (d) => d.y);

            labels.attr("x", (d) => d.x).attr("y", (d) => d.y);
        });

        // @ts-ignore
        function drag(simulation) {
            // @ts-ignore
            function dragstarted(event) {
                if (!event.active) simulation.alphaTarget(0.3).restart();
                event.subject.fx = event.subject.x;
                event.subject.fy = event.subject.y;
            }

            // @ts-ignore
            function dragged(event) {
                event.subject.fx = event.x;
                event.subject.fy = event.y;
            }

            // @ts-ignore
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
    });
</script>

<div class="graph-container">
    <h3>🧠 Multi-Modal Knowledge Graph</h3>
    <svg bind:this={svg} {width} {height}></svg>
    <div class="legend">
        <span><span class="dot text"></span> Text Node</span>
        <span><span class="dot image"></span> Image Node</span>
    </div>
</div>

<style>
    .graph-container {
        background: #1a1b26; /* Dark theme bg */
        border-radius: 8px;
        padding: 1rem;
        box-shadow: 0 4px 6px rgba(0, 0, 0, 0.3);
        margin: 1rem;
    }
    h3 {
        color: #a9a9b3;
        font-family: "Segoe UI", sans-serif;
        margin-bottom: 0.5rem;
    }
    svg {
        background: #24283b;
        border-radius: 4px;
        cursor: move;
    }
    .legend {
        display: flex;
        gap: 1rem;
        margin-top: 0.5rem;
        color: #a9a9b3;
        font-family: sans-serif;
        font-size: 0.9rem;
    }
    .dot {
        height: 10px;
        width: 10px;
        border-radius: 50%;
        display: inline-block;
        margin-right: 5px;
    }
    .text {
        background-color: #69db7c;
    }
    .image {
        background-color: #ff6b6b;
    }
</style>
