import { EditorView, keymap, hoverTooltip, Decoration, ViewPlugin } from "https://esm.sh/@codemirror/view";
import { EditorState, StateField } from "https://esm.sh/@codemirror/state";
import { defaultKeymap, history, historyKeymap } from "https://esm.sh/@codemirror/commands";
import { lineNumbers, highlightActiveLineGutter } from "https://esm.sh/@codemirror/view";
import { bracketMatching, indentOnInput } from "https://esm.sh/@codemirror/language";
import { closeBrackets } from "https://esm.sh/@codemirror/autocomplete";
import { oneDark } from "https://esm.sh/@codemirror/theme-one-dark";

import wasm from "./wasm.js"

function tokenize(text) {
    try{
        return wasm.lex(text);
    }catch(e){
        console.log(e)
        return []
    }
}

function compile(text) {
  try{
      return wasm.compile(text);
  }catch(e){
      console.log(e)
      return []
  }
}
/* ===================================================================== */

// Map token types -> CSS classes
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

// ===================== Diagnostics helpers =====================
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
function sevLabel(sev) {
  if (sev === "warning") return "warning";
  if (sev === "info") return "info";
  return "error";
}
function sevAnsiColorClass(sev) {
  if (sev === "warning") return "ansi-yellow";
  if (sev === "info") return "ansi-cyan";
  return "ansi-red";
}


function buildAnalysis(text, doc) {
  const tokens = tokenize(text);
  const {log, log_formatted} = compile(text);

  // Build ONE Decoration set: syntax + diagnostics
  const marks = [];
  const docLen = doc.length;

  for (const tok of tokens) {
    const start = Math.max(0, Math.min(docLen, tok.start));
    const end = Math.max(start, Math.min(docLen, tok.end));
    var tc = tokenClass(tok.kind);
    if (tc === "rb-"){
        tc += tok.scope_level.toString();
    }
    if (end > start) {
      marks.push(Decoration.mark({ class: tc }).range(start, end));
    }
  }

  for (const d of log) {
    if (d.start === undefined || d.end === undefined)continue;
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

  term.innerHTML = ansiToHtml(s+log_formatted);
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

// ===================== Build editor =====================
const initialText = `machine=NPDA
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
  const canvas = document.getElementById("canvas");
  const canvasPane = document.getElementById("canvasPane");

  let draggingH = false;
  let draggingV = false;

  // --- Canvas height splitter ---
  hSplit.addEventListener("mousedown", (e) => {
    draggingH = true;
    document.body.style.cursor = "row-resize";
    e.preventDefault();
  });

  // --- Terminal/editor width splitter ---
  vSplit.addEventListener("mousedown", (e) => {
    draggingV = true;
    document.body.style.cursor = "col-resize";
    e.preventDefault();
  });

  function resizeCanvasToPane() {
    // Keep canvas resolution in sync with CSS size (crisp rendering)
    const rect = canvasPane.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;
    const w = Math.max(1, Math.floor(rect.width * dpr));
    const h = Math.max(1, Math.floor(rect.height * dpr));
    if (canvas.width !== w) canvas.width = w;
    if (canvas.height !== h) canvas.height = h;

    // Optional: demo draw
    const ctx = canvas.getContext("2d");
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.fillStyle = "#1f6feb";
    ctx.fillRect(20 * dpr, 20 * dpr, 140 * dpr, 60 * dpr);
    ctx.fillStyle = "#c9d1d9";
    ctx.font = `${14 * dpr}px ui-monospace, monospace`;
    ctx.fillText("Canvas area", 30 * dpr, 55 * dpr);
  }

  window.addEventListener("mousemove", (e) => {
    const rect = app.getBoundingClientRect();

    if (draggingH) {
      const y = e.clientY - rect.top;
      const minCanvas = 80;
      const minBottom = 180;
      const maxCanvas = rect.height - 8 - minBottom;
      const canvasH = Math.max(minCanvas, Math.min(maxCanvas, y));
      app.style.setProperty("--canvasH", `${canvasH}px`);
      resizeCanvasToPane();
    }

    if (draggingV) {
      const bottomPane = document.getElementById("bottomPane");
      const r = bottomPane.getBoundingClientRect();
      const x = e.clientX - r.left;
      const minTerm = 220;
      const maxTerm = r.width - 8 - 220;
      const termW = Math.max(minTerm, Math.min(maxTerm, r.width-x));
      app.style.setProperty("--termW", `${termW}px`);
    }
  });

  window.addEventListener("mouseup", () => {
    draggingH = false;
    draggingV = false;
    document.body.style.cursor = "";
  });

  // Keep canvas crisp on window resize too
  window.addEventListener("resize", resizeCanvasToPane);
  resizeCanvasToPane();
})();