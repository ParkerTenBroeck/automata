// deno-lint-ignore-file no-unversioned-import

// deno-lint-ignore no-import-prefix
import * as vis from "npm:vis-network/standalone";
import { automaton, setAutomaton, sim } from "./automata.ts";

export const nodes = new vis.DataSet<vis.Node>();
export const edges = new vis.DataSet<vis.Edge>();

type Color = string;
type GraphTheme = {
  bg_0: Color;
  bg_1: Color;
  bg_2: Color;
  fg_0: Color;
  fg_1: Color;
  fg_2: Color;

  node_anchor: Color;
  node_border: Color;
  current_node_border: Color;

  edge: Color;
  edge_hover: Color;
  edge_active: Color;

  font_face: string

  node_font_size: number;
  node_font: string,
  node_font_bold: string,

  edge_font_size: number;
  edge_font: string,
  edge_font_bold: string,
};

let _graphTheme: GraphTheme | null = null;

function invalidateGraphThemeCache() {
  _graphTheme = null;
}

function getGraphTheme(): GraphTheme {
  function cssVar(name: string, fallback = ""): string {
    return getComputedStyle(document.documentElement)
      .getPropertyValue(name)
      .trim() || fallback;
  }

  if (_graphTheme) return _graphTheme;

  _graphTheme = {
    bg_0: cssVar("--graph-bg-0"),
    bg_1: cssVar("--graph-bg-1"),
    bg_2: cssVar("--graph-bg-2"),
    fg_0: cssVar("--graph-fg-0"),
    fg_1: cssVar("--graph-fg-1"),
    fg_2: cssVar("--graph-fg-2"),

    node_anchor: cssVar("--graph-node-anchor"),
    node_border: cssVar("--graph-node-border"),
    current_node_border: cssVar("--graph-current-node-border"),

    edge: cssVar("--graph-edge"),
    edge_hover: cssVar("--graph-edge-hover"),
    edge_active: cssVar("--graph-edge-active"),

    font_face: cssVar("--graph-font"),

    node_font_size: Number(cssVar("--graph-node-font-size")),
    node_font: `${cssVar("--graph-node-font-size")}px ${cssVar("--graph-font")}`,
    node_font_bold: `bold ${cssVar("--graph-node-font-size")}px ${cssVar("--graph-font")}`,

    edge_font_size: Number(cssVar("--graph-edge-font-size")),
    edge_font: `${Number(cssVar("--graph-edge-font-size"))}px ${cssVar("--graph-font")}`,
    edge_font_bold: `bold ${Number(cssVar("--graph-edge-font-size"))}px ${cssVar("--graph-font")}`,
  };

  return _graphTheme;
}

export function updateGraphTheme() {
  invalidateGraphThemeCache();
  const gt = getGraphTheme();

  network.setOptions({
    nodes: {
      labelHighlightBold: false,
      font: {
        color: gt.fg_0,
        bold: {
          color: gt.fg_1,
        },
      },
    },
    edges: {
      labelHighlightBold: true,
      font: {
        align: "top",
        face: gt.font_face,
        size: gt.edge_font_size,
        color: gt.fg_0,
        strokeColor: gt.bg_0,
        bold: {
          color: gt.fg_1,
          face: gt.font_face,
          size: gt.edge_font_size,
          mod: "bold",
        },
      },
      color: {
        color: gt.edge,
        hover: gt.edge_hover,
        highlight: gt.edge_active,
      },
      shadow: {
        enabled: false,
      },
    },
  });

  setAutomaton(automaton)
}


let _measureCanvas: HTMLCanvasElement | null = null;

export function measureTextWidth(text: string, font: string): number {
  if (!_measureCanvas) {
    _measureCanvas = document.createElement("canvas");
  }

  const ctx = _measureCanvas.getContext("2d")!;
  ctx.font = font;

  return ctx.measureText(text).width;
}

export function updateVisualization() {
  // Populate nodes
  for (const state of automaton.states.keys()) {
    
    const size = measureTextWidth(state, getGraphTheme().node_font)/2+10
    if (nodes.get(state)) {
      nodes.update({
        id: state,
        label: state,
        size
      });
    } else {
      nodes.add({
        id: state,
        label: state,
        size
      });
    }
  }

  // Populate edges
  for (const [edge_id, transitions] of automaton.edges) {
    const to_from = edge_id.split("#");
    const vadjust = -getGraphTheme().edge_font_size *
        Math.floor(transitions.length / 2);
    const font = {
      vadjust,
        bold: {
          vadjust
        }
    };    
    if (edges.get(edge_id)) {
      edges.update({
        id: edge_id,
        font,
        from: to_from[0],
        to: to_from[1],
        label: transitions.map(i => i.repr).join(automaton.type=="fa"?",":"\n"),
      });
    } else {
      edges.add({
        id: edge_id,
        font,
        from: to_from[0],
        to: to_from[1],
        label: transitions.map(i => i.repr).join(automaton.type=="fa"?",":"\n"),
      });
    }
  }

  for (const edge_id of edges.getIds()) {
    if (!automaton.edges.has(edge_id as string)) {
      edges.remove(edge_id);
    }
  }

  for (const node_id of nodes.getIds()) {
    if (!automaton.states.has(node_id as string)) {
      nodes.remove(node_id);
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

function renderNode({
  ctx,
  id,
  x,
  y,
  state: { selected, hover },
  style,
  label,
}: {ctx: CanvasRenderingContext2D, id: string, x: number, y: number, state: {selected: boolean, hover: boolean}, style: any, label: string}) {
  return {
    drawNode() {
      const t = getGraphTheme();
      const r = Math.max(14, style?.size ?? 18);

      const isInitial = automaton.initial_state === id;
      const isFinal = automaton.final_states
        ? automaton.final_states.has(id)
        : false;
      const isActive = sim?sim.current_states.has(id):false;

      const fill = selected ? t.bg_2 : hover ? t.bg_1 : t.bg_0;
      const stroke = isActive ? t.current_node_border : t.node_border;

      const emphasis = (selected ? 1 : 0) + (hover ? 0.6 : 0);

      const outerW = isFinal ? 3.5 : 3;
      const innerW = 2;

      ctx.save();

      ctx.font = hover||selected?t.node_font_bold:t.node_font;
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

      if (isActive) {
        const paths = sim?.current_states.get(id)!;
        const padX = 8;
        const padY = 6;
        const lineH = 14;

        let w = 0;
        for (const ln of paths) w = Math.max(w, ctx.measureText(ln.toString()).width);
        const boxW = w + padX * 2;
        const boxH = paths.length * lineH + padY * 2;

        const bx = x - boxW / 2;
        const by = y - r - 12 - boxH;

        ctx.fillStyle = t.bg_1;
        ctx.strokeStyle = t.bg_2;
        ctx.lineWidth = 1;
        roundRect(ctx, bx, by, boxW, boxH, 8);
        ctx.fill();
        ctx.stroke();

        ctx.textBaseline = "top";
        for (let i = 0; i < paths.length; i++) {
          ctx.fillStyle = paths[i].accepted?t.current_node_border:t.fg_0;
          ctx.fillText(paths[i].toString(), x, by + padY + i * lineH);
        }
      }

      const node: vis.Node = nodes.get(id)!;
      const physicsOff = node.physics === false;
      if (physicsOff) {
        drawPinIndicator(ctx, x, y, r, t.node_anchor);
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
