<script>
    // @ts-nocheck
    import { onMount } from "svelte";
    import * as d3 from "d3";
    import { invoke } from "@tauri-apps/api/core";

    /** @type {SVGSVGElement} */
    let svg;
    let width = 800;
    let height = 600;

    async function loadGraphData() {
        try {
            const data = await invoke("get_knowledge_graph");
            initGraph(data);
        } catch (e) {
            console.error("Failed to load graph:", e);
        }
    }

    function initGraph(data) {
        if (!svg) return;
        d3.select(svg).selectAll("*").remove();

        // @ts-ignore
        const nodes = data.nodes.map((d) => ({ ...d }));
        // @ts-ignore
        const links = data.edges.map((d) => ({ ...d }));

        const simulation = d3
            .forceSimulation(nodes)
            .force(
                "link",
                d3
                    .forceLink(links)
                    .id((d) => d.id)
                    .distance(150),
            )
            .force("charge", d3.forceManyBody().strength(-400))
            .force("center", d3.forceCenter(width / 2, height / 2))
            .force("collision", d3.forceCollide().radius(40));

        const svgEl = d3.select(svg).attr("viewBox", [0, 0, width, height]);

        // Definition for glow
        const defs = svgEl.append("defs");
        const filter = defs
            .append("filter")
            .attr("id", "glow")
            .attr("x", "-50%")
            .attr("y", "-50%")
            .attr("width", "200%")
            .attr("height", "200%");
        filter
            .append("feGaussianBlur")
            .attr("stdDeviation", "3.5")
            .attr("result", "coloredBlur");
        const feMerge = filter.append("feMerge");
        feMerge.append("feMergeNode").attr("in", "coloredBlur");
        feMerge.append("feMergeNode").attr("in", "SourceGraphic");

        // Edges
        const link = svgEl
            .append("g")
            .selectAll("line")
            .data(links)
            .join("line")
            .attr("stroke", "rgba(0, 255, 204, 0.3)")
            .attr("stroke-width", 2);

        // Nodes
        const node = svgEl
            .append("g")
            .selectAll("g")
            .data(nodes)
            .join("g")
            .call(drag(simulation));

        node.append("circle")
            .attr("r", 15)
            .attr("fill", (d) => {
                if (d.type === "human") return "#0080ff";
                if (d.type === "agent") return "#00ffcc";
                return "#a29bfe";
            })
            .style("filter", "url(#glow)");

        node.append("text")
            .attr("dy", 30)
            .attr("text-anchor", "middle")
            .text((d) => d.id)
            .style("fill", "#fff")
            .style("font-size", "12px")
            .style("font-family", "'Outfit', 'Inter', sans-serif")
            .style("text-shadow", "0 0 5px rgba(0,0,0,0.8)");

        simulation.on("tick", () => {
            link.attr("x1", (d) => d.source.x)
                .attr("y1", (d) => d.source.y)
                .attr("x2", (d) => d.target.x)
                .attr("y2", (d) => d.target.y);

            node.attr("transform", (d) => `translate(${d.x},${d.y})`);
        });

        function drag(simulation) {
            function dragstarted(event) {
                if (!event.active) simulation.alphaTarget(0.3).restart();
                event.subject.fx = event.subject.x;
                event.subject.fy = event.subject.y;
            }
            function dragged(event) {
                event.subject.fx = event.x;
                event.subject.fy = event.y;
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
    }

    onMount(() => {
        loadGraphData();
    });
</script>

<div class="graph-container">
    <svg bind:this={svg}></svg>
</div>

<style>
    .graph-container {
        width: 100%;
        height: 100%;
        display: flex;
        justify-content: center;
        align-items: center;
        background: radial-gradient(
            circle at center,
            rgba(10, 10, 42, 0.5) 0%,
            rgba(0, 0, 0, 0) 100%
        );
    }

    svg {
        width: 100%;
        height: 100%;
        max-width: 800px;
        max-height: 600px;
    }
</style>
