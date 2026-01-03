<script>
    import { onMount } from "svelte";
    import * as d3 from "d3";
    import { invoke } from "@tauri-apps/api/core";
    import { toast } from "@zerodevx/svelte-toast";

    /** @type {SVGSVGElement} */
    let svg;
    let width = 800;
    let height = 600;

    onMount(async () => {
        try {
            let data;
            // @ts-ignore
            if (window.__TAURI__) {
                data = await invoke('get_knowledge_graph');
            } else {
                // Fallback to static data in browser mode
                const response = await fetch("/knowledge_graph.json");
                data = await response.json();
            }

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

            // Add zoom
            const zoom = d3.zoom()
                .scaleExtent([0.1, 4])
                .on("zoom", (event) => {
                // @ts-ignore
                svgEl.attr("transform", event.transform);
            });

        // @ts-ignore
        svgEl.call(zoom);

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

        // Fit to view after simulation stabilizes
        simulation.on('end', () => {
            // @ts-ignore
            const bounds = svgEl.node().getBBox();
            const fullWidth = bounds.width;
            const fullHeight = bounds.height;
            const midX = bounds.x + fullWidth / 2;
            const midY = bounds.y + fullHeight / 2;

            const scale = 0.9 / Math.max(fullWidth / width, fullHeight / height);
            const translate = [width / 2 - scale * midX, height / 2 - scale * midY];

            // @ts-ignore
            svgEl.transition().duration(750).call(
                // @ts-ignore
                zoom.transform,
                // @ts-ignore
                d3.zoomIdentity.translate(translate[0], translate[1]).scale(scale)
            );
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
        } catch (e) {
            console.error('Failed to load graph:', e);
            toast.push(`Failed to load graph: ${e}`);
        }
    });
</script>

<div class="graph-container">
    <svg bind:this={svg} {width} {height}></svg>
</div>

<style>
    .graph-container {
        width: 100%;
        height: 100%;
        background: transparent;
        position: relative;
    }

    svg {
        width: 100%;
        height: 100%;
        background: transparent;
        border-radius: 8px;
    }
</style>
