// deno-lint-ignore-file no-unversioned-import

// deno-lint-ignore no-import-prefix
import * as vis from "npm:vis-network/standalone";



const nodes = new vis.DataSet<vis.Node>();
const edges = new vis.DataSet<vis.Edge>();


const automaton = {
  states: ["q0", "q1"],
  initialState: "q0",
  acceptStates: ["q1"],

  transitions: [
    {
      from: "q0",
      to: "q0",
      label: "ε, z0 → A z0\n"
    },
    {
      from: "q0",
      to: "q0",
      label: "ε, z0 → B z0"
    },
    {
      from: "q0",
      to: "q1",
      label: "ε, z0 → z0"
    },
    {
      from: "q1",
      to: "q1",
      label: "a, A → ε"
    },
    {
      from: "q1",
      to: "q1",
      label: "b, B → ε"
    }
  ]
};

function renderNode({
  ctx,
  id,
  x,
  y,
  state: { selected, hover },
  style,
  label,
}: any) {
  return {
    drawNode() {
      ctx.save();
      const r = style.size;


      ctx.beginPath();
      ctx.arc(x, y, r, 0, 2 * Math.PI);
      ctx.fillStyle = "red";
      ctx.fill();
      ctx.lineWidth = 4;
      ctx.strokeStyle = "blue";
      ctx.stroke();

      ctx.fillStyle = "black";
      ctx.textAlign = 'center';
      ctx.fillText(label, x, y, r);


      ctx.textAlign = 'center';
      ctx.strokeStyle = 'white';
      ctx.fillStyle = "black";
      let cy = y - (r + 10);
      for (const part of "meow[]\nbeeep".split("\n").reverse()) {
        const metrics = ctx.measureText(part);
        cy -= metrics.actualBoundingBoxAscent + metrics.actualBoundingBoxDescent;
        ctx.strokeText(part, x, cy);
        ctx.fillText(part, x, cy);
      }


      ctx.restore();
    },
    nodeDimensions: { width: 20, height: 20 },
  };
}


// Populate nodes
for (const state of automaton.states) {
  nodes.add({
    id: state,
    label: state,
  });
}

// Populate edges
automaton.transitions.forEach((t, i) => {
  edges.add({
    id: `e${i}`,
    from: t.from,
    to: t.to,
    label: t.label
  });
});


function chosen_edge(_: vis.ChosenNodeValues, id: vis.IdType,selected: boolean, hovered: boolean) {
  console.log("edge", id, selected, hovered)
}

function chosen_node(_: vis.ChosenNodeValues, id: vis.IdType,selected: boolean, hovered: boolean) {
  console.log("node", id, selected, hovered)
}


const network: vis.Network = createGraph();

function createGraph(): vis.Network {

  const container = document.getElementById("graph")!;

  const network = new vis.Network(
    container,
    { nodes, edges },
    {
      layout: { improvedLayout: true },
      autoResize: true,
      width: "99%",
      physics: {
        enabled: true,
        solver: "barnesHut",
        barnesHut: { gravitationalConstant: -8000, springLength: 120, springConstant: 0.04 },
        stabilization: { iterations: 200 }
      },
      interaction: {
        dragNodes: true,
        hover: true,
        multiselect: true,
        hoverConnectedEdges: false,
        selectConnectedEdges: false,
      },
      nodes: {
        font: { color: "#c9d1d9" },
        color: {
          background: "#1f6feb",
          border: "#79c0ff",
          highlight: { background: "#388bfd", border: "#a5d6ff" }
        },
        // @ts-expect-error  bad library
        chosen: {
          node: chosen_node,
        },
        shape: "custom",
        ctxRenderer: renderNode,
        size: 18,
      },
      edges: {
        chosen: {
          // @ts-expect-error bad library
          edge: chosen_edge
        },
        arrowStrikethrough: false,
        font: { align: "middle", color: "#000000ff" },
        color: { color: "rgba(201,209,217,0.35)", highlight: "#c9d1d9" },
        // @ts-expect-error  bad library
        smooth: { type: "dynamic" },
        arrows: "to",
      }
    }
  );
  vis.DataSet

  network.on("doubleClick", (params: any) => {
    
    for (const node_id of params.nodes){
      // @ts-expect-error bad library
      const node: vis.Node = nodes.get(node_id)!;
      node.physics = !node.physics;
      nodes.update(node)
    }
  });
  
  return network;
}
