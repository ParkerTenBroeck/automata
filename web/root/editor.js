import { EditorView, keymap, hoverTooltip, Decoration, ViewPlugin } from "https://esm.sh/@codemirror/view";
import { EditorState, StateField } from "https://esm.sh/@codemirror/state";
import { defaultKeymap, history, historyKeymap } from "https://esm.sh/@codemirror/commands";
import { lineNumbers, highlightActiveLineGutter } from "https://esm.sh/@codemirror/view";
import { bracketMatching, indentOnInput } from "https://esm.sh/@codemirror/language";
import { closeBrackets } from "https://esm.sh/@codemirror/autocomplete";
import { oneDark } from "https://esm.sh/@codemirror/theme-one-dark";

import wasm from "./wasm.js"

import * as vis from "./js/vis-network.js"


function tokenize(text) {
  try {
    return wasm.lex(text);
  } catch (e) {
    console.log(e)
    return []
  }
}

function compile(text) {
  try {
    return wasm.compile(text);
  } catch (e) {
    console.log(e)
    return []
  }
}

const tokenClass = (t) =>
({
  comment: "tok-comment",
  keyword: "tok-keyword",
  error: "tok-error",
  ident: "tok-ident",
  punc: "tok-punc",
  string: "tok-string",
  lpar: "rb-",
  lbrace: "rb-",
  lbracket: "rb-",

  rpar: "rb-",
  rbrace: "rb-",
  rbracket: "rb-",
}[t] || "tok-ident");


function severityClass(sev) {
  const s = (sev || "error").toLowerCase();
  if (s === "warning") return "cm-diag-warning";
  if (s === "info") return "cm-diag-info";
  return "cm-diag-error";
}
function sevRank(sev) {
  if (sev === "error") return 3;
  if (sev === "warning") return 2;
  return 1;
}


function buildAnalysis(text, doc) {
  const tokens = tokenize(text);
  const { log, log_formatted } = compile(text);

  // Build ONE Decoration set: syntax + diagnostics
  const marks = [];
  const docLen = doc.length;

  for (const tok of tokens) {
    const start = Math.max(0, Math.min(docLen, tok.start));
    const end = Math.max(start, Math.min(docLen, tok.end));
    var tc = tokenClass(tok.kind);
    if (tc === "rb-") {
      tc += tok.scope_level.toString();
    }
    if (end > start) {
      marks.push(Decoration.mark({ class: tc }).range(start, end));
    }
  }

  for (const d of log) {
    if (d.start === undefined || d.end === undefined) continue;
    const start = Math.max(0, Math.min(docLen, d.start));
    const endRaw = d.end == null ? d.start : d.end;
    const end = Math.max(start, Math.min(docLen, endRaw));
    const cls = severityClass(d.level);
    if (end > start) {
      marks.push(Decoration.mark({ class: cls }).range(start, end));
    } else {
      const end = Math.min(docLen, start + 1);
      if (end > start) marks.push(Decoration.mark({ class: cls }).range(start, end));
    }
  }

  const deco = Decoration.set(marks, true);
  return { tokens, log, log_formatted, deco };
}

const analysisField = StateField.define({
  create(state) {
    const text = state.doc.toString();
    return buildAnalysis(text, state.doc);
  },
  update(value, tr) {
    if (!tr.docChanged) return value;
    const text = tr.state.doc.toString();
    return buildAnalysis(text, tr.state.doc);
  },
  provide: (f) => EditorView.decorations.from(f, (v) => v.deco),
});

// ===================== Hover tooltip (uses cached diags) =====================
const diagHover = hoverTooltip((view, pos) => {
  const { log } = view.state.field(analysisField);
  const hits = log.filter((d) => pos >= d.start && pos <= d.end);
  if (hits.length === 0) return null;

  const top = hits.reduce((a, b) => (sevRank(b.level) > sevRank(a.level) ? b : a), hits[0]);

  return {
    pos,
    end: pos,
    above: true,
    create() {
      const dom = document.createElement("div");
      dom.className = "cm-tooltip cm-tooltip-hover";

      const title = document.createElement("div");
      title.className = `tipTitle ${top.level}`;
      title.textContent =
        hits.length === 1 ? top.level.toUpperCase() : `${top.level.toUpperCase()} (${hits.length})`;

      const body = document.createElement("div");
      body.className = "tipBody";
      body.textContent = hits
        .slice()
        .sort((a, b) => sevRank(b.level) - sevRank(a.level))
        .map((h) => `[${h.level.toUpperCase()}] ${h.message}`)
        .join("\n");

      dom.appendChild(title);
      dom.appendChild(body);
      return { dom };
    },
  };
});


function escapeHtml(s) {
  return String(s)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}


function ansiToHtml(input) {
  const ESC_RE = /\x1b\[([0-9;]*)m/g;

  let out = "";
  let lastIndex = 0;

  // current style state
  let fg = null; // e.g. 31, 92
  let bg = null; // e.g. 41
  let bold = false;
  let dim = false;

  function openSpanIfNeeded(text) {
    if (text.length === 0) return "";
    const classes = [];
    if (bold) classes.push("ansi-bold");
    if (dim) classes.push("ansi-dim");
    if (fg != null) classes.push(`ansi-fg-${fg}`);
    if (bg != null) classes.push(`ansi-bg-${bg}`);
    if (classes.length === 0) return escapeHtml(text);
    return `<span class="${classes.join(" ")}">${escapeHtml(text)}</span>`;
  }

  function applyCodes(codes) {
    if (codes.length === 0) codes = [0];
    for (const c of codes) {
      const code = Number(c);
      if (Number.isNaN(code)) continue;

      if (code === 0) {
        fg = null; bg = null; bold = false; dim = false;
      } else if (code === 1) {
        bold = true;
      } else if (code === 2) {
        dim = true;
      } else if (code === 22) {
        bold = false; dim = false;
      } else if (code === 39) {
        fg = null;
      } else if (code === 49) {
        bg = null;
      } else if ((code >= 30 && code <= 37) || (code >= 90 && code <= 97)) {
        fg = code;
      } else if ((code >= 40 && code <= 47) || (code >= 100 && code <= 107)) {
        bg = code;
      }
    }
  }

  let m;
  while ((m = ESC_RE.exec(input)) !== null) {
    const chunk = input.slice(lastIndex, m.index);
    out += openSpanIfNeeded(chunk);

    const codes = m[1] ? m[1].split(";") : [];
    applyCodes(codes);

    lastIndex = ESC_RE.lastIndex;
  }

  out += openSpanIfNeeded(input.slice(lastIndex));
  return out;
}

function formatTerminal(view) {
  const term = document.getElementById("terminal");
  if (!term) return;

  const { log, log_formatted } = view.state.field(analysisField);

  let s = "";
  s += `\x1b[90m[compile]\x1b[0m ${log.length} diagnostics\n`;

  term.innerHTML = ansiToHtml(s + log_formatted);
}

const terminalPlugin = ViewPlugin.fromClass(
  class {
    constructor(view) {
      this.view = view;
      formatTerminal(view);
    }
    update(update) {
      if (update.docChanged) formatTerminal(update.view);
    }
  }
);


const initialText = `type=NPDA
Q = {q0, q1} // states
E = {a, b} // alphabet
T = {z0, A, B} // stack
q0 = q0
z0 = z0

// construct all possible permutations of A's and B's
d(q0, epsilon, z0)  =   { (q0, [A z0]), (q0, [B z0]) }
d(q0, epsilon, A)   =   { (q0, [A A]),  (q0, [B A])  }

d(q0, epsilon, B)   =   { (q0, [A B]),  (q0, [B B])  }

// transition to q1
d(q0, epsilon, z0)  =   { (q1, z0) }
d(q0, epsilon, A)   =   { (q1, A)  }
d(q0, epsilon, B)   =   { (q1, B)  }

// consume stack until empty
d(q1, a, A)         =   { (q1, epsilon) }
d(q1, b, B)         =   { (q1, epsilon) }
`;

const state = EditorState.create({
  doc: initialText,
  extensions: [
    lineNumbers(),
    highlightActiveLineGutter(),
    history(),
    indentOnInput(),
    bracketMatching(),
    closeBrackets(),
    keymap.of([...defaultKeymap, ...historyKeymap]),
    oneDark,

    analysisField,
    diagHover,
    terminalPlugin,

    EditorView.lineWrapping,
  ],
});

window.editor = new EditorView({
  state,
  parent: document.getElementById("editor"),
});


function setDefaultLayoutWeights() {
  const vh = window.innerHeight;
  const vw = window.innerWidth;

  // Canvas: 30% of screen height
  const canvasH = Math.round(vh * 0.60);

  // Terminal: 35% of width
  const termW = Math.round(vw * 0.30);

  const app = document.getElementById("app");
  app.style.setProperty("--canvasH", `${canvasH}px`);
  app.style.setProperty("--termW", `${termW}px`);
}

setDefaultLayoutWeights();


(function enableLayoutSplitters() {
  const app = document.getElementById("app");
  const hSplit = document.getElementById("hSplit");
  const vSplit = document.getElementById("vSplit");
  // const canvas = document.getElementById("canvas");
  const canvasPane = document.getElementById("canvasPane");

  let draggingH = false;
  let draggingV = false;

  hSplit.addEventListener("mousedown", (e) => {
    draggingH = true;
    document.body.style.cursor = "row-resize";
    e.preventDefault();
  });

  vSplit.addEventListener("mousedown", (e) => {
    draggingV = true;
    document.body.style.cursor = "col-resize";
    e.preventDefault();
  });

  window.addEventListener("mousemove", (e) => {
    const rect = app.getBoundingClientRect();

    if (draggingH) {
      const y = e.clientY - rect.top;
      const minCanvas = 80;
      const minBottom = 180;
      const maxCanvas = rect.height - 8 - minBottom;
      const canvasH = Math.max(minCanvas, Math.min(maxCanvas, y));
      app.style.setProperty("--canvasH", `${canvasH}px`);
    }

    if (draggingV) {
      const bottomPane = document.getElementById("bottomPane");
      const r = bottomPane.getBoundingClientRect();
      const x = e.clientX - r.left;
      const minTerm = 220;
      const maxTerm = r.width - 8 - 220;
      const termW = Math.max(minTerm, Math.min(maxTerm, r.width - x));
      app.style.setProperty("--termW", `${termW}px`);
    }
  });

  window.addEventListener("mouseup", () => {
    draggingH = false;
    draggingV = false;
    document.body.style.cursor = "";
  });
})();

let network = null;
const nodes = new vis.DataSet();
const edges = new vis.DataSet();


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

/**@param {{ctx: CanvasRenderingContext2D}} */
function renderNode({
  ctx,
  id,
  x,
  y,
  state: { selected, hover },
  style,
  label,
}) {
  return {
    drawNode() {
      ctx.save();
      var r = style.size;


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

// updateGraphFromText();
ensureGraph();
function updateGraphFromText() {
  ensureGraph();

  const trans = []

  // Collect state ids
  const stateSet = new Set();
  for (const tr of trans) {
    stateSet.add(tr.from);
    stateSet.add(tr.to);
  }

  // Update nodes (add missing, remove stale)
  const existingNodeIds = new Set(nodes.getIds());
  const desiredNodeIds = new Set([...stateSet]);

  // remove stale
  for (const id of existingNodeIds) {
    if (!desiredNodeIds.has(id)) nodes.remove(id);
  }
  // add/update desired
  for (const id of desiredNodeIds) {
    const pos = pinnedPositions.get(id);
    if (!existingNodeIds.has(id)) {
      nodes.add({
        id,
        label: id,
        ...(pos ? { x: pos.x, y: pos.y, fixed: true } : {})
      });
    } else if (pos) {
      nodes.update({ id, x: pos.x, y: pos.y, fixed: true });
    }
  }

  // Update edges (stable IDs so edits don't flicker)
  const desiredEdgeIds = new Set();
  const nextEdges = [];

  for (let i = 0; i < trans.length; i++) {
    const tr = trans[i];
    const id = `${tr.from}::${tr.to}::${tr.label}::${i}`;
    desiredEdgeIds.add(id);
    nextEdges.push({ id, from: tr.from, to: tr.to, label: tr.label });
  }

  const existingEdgeIds = new Set(edges.getIds());
  for (const id of existingEdgeIds) {
    if (!desiredEdgeIds.has(id)) edges.remove(id);
  }
  // add/update in batch
  for (const e of nextEdges) {
    if (!existingEdgeIds.has(e.id)) edges.add(e);
    else edges.update(e);
  }

  // If positions exist for all nodes, we can disable physics to “respect” manual layout
  // Otherwise leave physics on to auto-layout new nodes.
  const allPinned = [...desiredNodeIds].every((id) => pinnedPositions.has(id));
  network.setOptions({ physics: { enabled: !allPinned } });

  // Redraw nicely after updates
  network.fit({ animation: { duration: 200, easingFunction: "easeInOutQuad" } });
}

// ---------- 4) Hook graph updates into your existing single-pass analysis ----------
const graphPlugin = ViewPlugin.fromClass(class {
  constructor(view) {
    updateGraphFromText(view.state.doc.toString());
  }
  update(update) {
    if (update.docChanged) {
      updateGraphFromText(update.state.doc.toString());
    }
  }
});

function chosen_node(values, id, selected, hovering) {
  
  console.log(values, id, selected, hovering)
}

function ensureGraph() {
  if (network) return;

  const container = document.getElementById("graph");
  network = new vis.Network(
    container,
    { nodes, edges },
    {
      layout: { improvedLayout: true },
      physics: {
        enabled: true,
        solver: "barnesHut",
        barnesHut: { gravitationalConstant: -8000, springLength: 120, springConstant: 0.04 },
        stabilization: { iterations: 200 }
      },
      interaction: {
        dragNodes: true,
        hover: true,
        multiselect: true
      },
      nodes: {
        shape: 'dot',
        size: 14,
        font: { color: "#c9d1d9" },
        color: {
          background: "#1f6feb",
          border: "#79c0ff",
          highlight: { background: "#388bfd", border: "#a5d6ff" }
        },
        chosen: {
          node: chosen_node
        },
        shape: "custom",
        ctxRenderer: renderNode,
        size: 18,
      },
      edges: {
        arrows: { to: { enabled: true, scaleFactor: 0.8 } },
        arrowStrikethrough: false,
        font: { align: "middle", color: "#000000ff" },
        color: { color: "rgba(201,209,217,0.35)", highlight: "#c9d1d9" },
        smooth: { type: "dynamic" },
        arrows: "to",
      }
    }
  );

  // Save positions when user drags nodes

  network.on("dragEnd", (params) => {
    const pos = network.getPositions(params.nodes);
    for (const id of params.nodes) {
      pinnedPositions.set(id, pos[id]);
    }
  });

  window.network = network;
}
