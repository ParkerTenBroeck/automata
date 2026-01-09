// deno-lint-ignore-file no-unversioned-import

// deno-lint-ignore no-import-prefix
import * as vis from "npm:vis-network/standalone";

export const nodes = new vis.DataSet<vis.Node>();
export const edges = new vis.DataSet<vis.Edge>();

type StateId = string;
type GraphDef = {
  initial: StateId;
  final: StateId[];
  states: StateId[];
  transitions: Record<string, string>;
};

let automaton: GraphDef = {
  initial: "",
  final: [],
  states: [],
  transitions: {},
};

export function clearAutomaton() {
  setAutomaton({
    initial: "",
    final: [],
    states: [],
    transitions: {},
  });
}

export function setAutomaton(auto: GraphDef) {
  automaton = auto;
  // Populate nodes
  for (const state of automaton.states) {
    if (nodes.get(state)) {
      nodes.update({
        id: state,
        label: state,
      });
    } else {
      nodes.add({
        id: state,
        label: state,
      });
    }
  }

  // Populate edges
  for (const [k, v] of Object.entries(automaton.transitions)) {
    const to_from = k.split("#");
    const font = {
      vadjust: -getGraphTheme().edge_font_size*Math.floor(((v.match(/\n/g) || '').length + 1)/2)
    };
    if (edges.get(k)) {
      edges.update({
        id: k,
        font,
        from: to_from[0],
        to: to_from[1],
        label: v,
      });
    } else {
      edges.add({
        id: k,
        font,
        from: to_from[0],
        to: to_from[1],
        label: v,
      });
    }
  }

  for (const edge_id of edges.getIds()){
    if (auto.transitions[edge_id as string] === undefined){
      edges.remove(edge_id)
    }
  }

  for (const node_id of nodes.getIds()){
    if (!auto.states.includes(node_id as string)){
      nodes.remove(node_id)
    }
  }
}

function chosen_edge(
  _: vis.ChosenNodeValues,
  id: vis.IdType,
  selected: boolean,
  hovered: boolean,
) {
}

function chosen_node(
  _: vis.ChosenNodeValues,
  id: vis.IdType,
  selected: boolean,
  hovered: boolean,
) {
}

export const network: vis.Network = createGraph();

function createGraph(): vis.Network {
  const container = document.getElementById("graph")!;

  const network = new vis.Network(
    container,
    { nodes, edges },
    {
      layout: { improvedLayout: true },
      physics: {
        enabled: true,
        solver: "forceAtlas2Based",
        // solver: "barnesHut",
        // barnesHut: {
        //   gravitationalConstant: -8000,
        //   springLength: 120,
        //   springConstant: 0.04,
        // },
        // stabilization: { iterations: 200 },
      },
      interaction: {
        dragNodes: true,
        hover: true,
        multiselect: true,
        hoverConnectedEdges: false,
        selectConnectedEdges: false,
      },
      nodes: {
        shape: "custom",
        size: 18,
        // // @ts-expect-error  bad library
        // chosen: {
        //   node: chosen_node,
        // },
        // @ts-expect-error  bad library
        ctxRenderer: renderNode,
      },
      edges: {
        chosen: {
          // // @ts-expect-error bad library
          // edge: chosen_edge,
        },
        arrowStrikethrough: false,
        arrows: "to",
      },
    },
  );
  vis.DataSet;

  network.on("doubleClick", (params: any) => {
    for (const node_id of params.nodes) {
      // @ts-expect-error bad library
      const node: vis.Node = nodes.get(node_id)!;
      node.physics = !node.physics;
      nodes.update(node);
    }
  });

  return network;
}

export type GraphTheme = {
  bg_0: string;
  bg_1: string;
  bg_2: string;
  fg_0: string;

  anchor: string;
  selected: string;
  node: string;
  current: string;
  edge: string;
  glow: string;
  edge_font_size: number,
};

let _graphTheme: GraphTheme | null = null;

export function invalidateGraphThemeCache() {
  _graphTheme = null;
}

export function getGraphTheme(): GraphTheme {
  function cssVar(name: string, fallback = ""): string {
    return getComputedStyle(document.documentElement)
      .getPropertyValue(name)
      .trim() || fallback;
  }

  if (_graphTheme) return _graphTheme;

  _graphTheme = {
    bg_0: cssVar("--bg-0"),
    bg_1: cssVar("--bg-1"),
    bg_2: cssVar("--bg-2"),
    fg_0: cssVar("--fg-0"),

    selected: cssVar("--bg-2"),

    node: cssVar("--focus"),
    current: cssVar("--success"),

    anchor: cssVar("--warning"),

    edge: cssVar("--graph-edge", "rgba(201,209,217,0.55)"),

    glow: cssVar("--accent", "#79c0ff"),
    edge_font_size: 10,
  };

  return _graphTheme;
}

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
      // @ts-expect-error bad library
      const node: vis.Node = nodes.get(id)!;

      const t = getGraphTheme();
      const r = Math.max(14, style?.size ?? 18);

      const isInitial = id === "q0";
      const isFinal = id === "q1"; // <-- change if your schema differs
      const isActive = id === "q0"; // <-- change if your schema differs

      const fill = selected ? t.glow : hover ? t.bg_1 : t.bg_0;
      const stroke = isActive ? t.current : t.node;

      const emphasis = (selected ? 1 : 0) + (hover ? 0.6 : 0);

      const outerW = isFinal ? 3.5 : 3;
      const innerW = 2;

      ctx.save();

      ctx.textAlign = "center";
      ctx.textBaseline = "middle";

      ctx.lineWidth = outerW + emphasis;
      ctx.strokeStyle = stroke;
      ctx.fillStyle = fill;
      ctx.beginPath();
      ctx.arc(x, y, r - ctx.lineWidth * 0.5, 0, Math.PI * 2);
      ctx.stroke();
      ctx.fill();

      if (isFinal) {
        ctx.lineWidth = innerW;
        ctx.strokeStyle = stroke;
        ctx.beginPath();
        ctx.arc(x, y, r - 7, 0, Math.PI * 2);
        ctx.stroke();
      }

      ctx.lineWidth = 2;
      ctx.fillStyle = t.fg_0;
      ctx.strokeStyle = t.bg_0;
      ctx.strokeText(label, x, y);
      ctx.fillText(label, x, y);

      if (isInitial) {
        drawInitialArrow(ctx, x, y, r, t.edge);
      }

      // const badgeText = "bleh\npee";
      // if (badgeText) {
      //   const lines = badgeText.split("\n").slice(0, 3);
      //   const padX = 8;
      //   const padY = 6;
      //   const lineH = 14;

      //   let w = 0;
      //   for (const ln of lines) w = Math.max(w, ctx.measureText(ln).width);
      //   const boxW = w + padX * 2;
      //   const boxH = lines.length * lineH + padY * 2;

      //   const bx = x - boxW / 2;
      //   const by = y - r - 12 - boxH;

      //   ctx.fillStyle = t.bg_1;
      //   ctx.strokeStyle = t.bg_2;
      //   ctx.lineWidth = 1;
      //   roundRect(ctx, bx, by, boxW, boxH, 8);
      //   ctx.fill();
      //   ctx.stroke();

      //   ctx.fillStyle = t.fg_0;
      //   ctx.textBaseline = "top";
      //   for (let i = 0; i < lines.length; i++) {
      //     ctx.fillText(lines[i], x, by + padY + i * lineH);
      //   }
      // }

      const physicsOff = node.physics === false;
      if (physicsOff) {
        drawPinIndicator(ctx, x, y, r, t.anchor);
      }

      ctx.restore();
    },
  };
}

function drawInitialArrow(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  r: number,
  color: string,
) {
  const len = Math.max(14, r * 0.95); // arrow length
  const head = Math.max(7, r * 0.32); // arrow head size
  const lineW = Math.max(2, r * 0.12); // stroke width
  const gap = 4; // distance from node edge

  // Direction: from top-left → center (45° down-right)
  const dx = Math.SQRT1_2;
  const dy = Math.SQRT1_2;

  // Tip position (just outside node)
  const tipX = x - dx * (r + gap);
  const tipY = y - dy * (r + gap);

  // Tail start
  const tailX = tipX - dx * len;
  const tailY = tipY - dy * len;

  // Perpendicular for arrow head
  const px = -dy;
  const py = dx;

  ctx.save();

  ctx.lineCap = "round";
  ctx.lineJoin = "round";
  ctx.lineWidth = lineW;
  ctx.strokeStyle = color;
  ctx.fillStyle = color;

  // Shaft
  ctx.beginPath();
  ctx.moveTo(tailX, tailY);
  ctx.lineTo(
    tipX - dx * head * 0.6,
    tipY - dy * head * 0.6,
  );
  ctx.stroke();

  // Head
  ctx.beginPath();
  ctx.moveTo(tipX, tipY);
  ctx.lineTo(
    tipX - dx * head + px * head * 0.7,
    tipY - dy * head + py * head * 0.7,
  );
  ctx.lineTo(
    tipX - dx * head - px * head * 0.7,
    tipY - dy * head - py * head * 0.7,
  );
  ctx.closePath();
  ctx.fill();

  ctx.restore();
}

function drawPinIndicator(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  r: number,
  color: string,
) {
  const size = Math.max(7, Math.round(r * 0.28));
  const ox = x + r - size * 0.55;
  const oy = y + r - size * 0.55;

  const stroke = color;
  const fill = "rgba(0,0,0,0)";

  ctx.save();

  ctx.shadowColor = "rgba(0,0,0,0)";
  ctx.shadowBlur = 6;
  ctx.shadowOffsetX = 0;
  ctx.shadowOffsetY = 2;

  // Pin head (circle)
  ctx.beginPath();
  ctx.arc(ox, oy, size * 0.55, 0, Math.PI * 2);
  ctx.fillStyle = fill;
  ctx.fill();

  // Pin stem (triangle-ish)
  ctx.beginPath();
  ctx.moveTo(ox, oy + size * 0.25);
  ctx.lineTo(ox - size * 0.35, oy + size * 0.95);
  ctx.lineTo(ox + size * 0.35, oy + size * 0.95);
  ctx.closePath();
  ctx.fillStyle = fill;
  ctx.fill();

  // Outline
  ctx.shadowBlur = 0;
  ctx.lineWidth = Math.max(1.25, Math.round(r * 0.06));
  ctx.strokeStyle = stroke;

  ctx.beginPath();
  ctx.arc(ox, oy, size * 0.55, 0, Math.PI * 2);
  ctx.stroke();

  ctx.beginPath();
  ctx.moveTo(ox, oy + size * 0.25);
  ctx.lineTo(ox - size * 0.35, oy + size * 0.95);
  ctx.lineTo(ox + size * 0.35, oy + size * 0.95);
  ctx.closePath();
  ctx.stroke();

  // Inner dot
  ctx.beginPath();
  ctx.arc(ox, oy, size * 0.18, 0, Math.PI * 2);
  ctx.fillStyle = stroke;
  ctx.fill();

  ctx.restore();
}

// Small helper for rounded rectangles
function roundRect(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  w: number,
  h: number,
  r: number,
) {
  const rr = Math.min(r, w / 2, h / 2);
  ctx.beginPath();
  ctx.moveTo(x + rr, y);
  ctx.arcTo(x + w, y, x + w, y + h, rr);
  ctx.arcTo(x + w, y + h, x, y + h, rr);
  ctx.arcTo(x, y + h, x, y, rr);
  ctx.arcTo(x, y, x + w, y, rr);
  ctx.closePath();
}
