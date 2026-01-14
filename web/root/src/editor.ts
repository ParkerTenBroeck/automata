// deno-lint-ignore-file

import {
  Decoration,
  DecorationSet,
  EditorView,
  highlightActiveLine,
  highlightActiveLineGutter,
  hoverTooltip,
  keymap,
  lineNumbers,
} from "npm:@codemirror/view";

import { EditorState, RangeSetBuilder, StateEffect, StateField, Text } from "npm:@codemirror/state";
import {
  defaultKeymap,
  history,
  historyKeymap,
} from "npm:@codemirror/commands";
import { bracketMatching, indentOnInput } from "npm:@codemirror/language";
import { closeBrackets } from "npm:@codemirror/autocomplete";

import wasm from "./wasm.ts";

import { Share } from "./share.ts";
import { examples } from "./examples.ts";
import { bus } from "./bus.ts";
import { current, Highlight, highlight_span_attr, HighlightKind } from "./highlight.ts";
import { Machine, parse_machine_from_json, Span } from "./automata.ts";

function tokenize(text: string): wasm.Tok[] {
  try {
    return wasm.lex(text);
  } catch (e) {
    console.log(e);
    return [];
  }
}

function compile(
  text: string,
): { log: wasm.CompileLog[]; ansi_log: string; machine: Machine | undefined } {
  try {
    const res = wasm.compile(text);
    return {machine: res.machine ? parse_machine_from_json(res.machine):undefined, log: res.log, ansi_log: res.ansi_log};
  } catch (e) {
    console.log(e);
    return { log: [], ansi_log: "", machine: undefined };
  }
}


const eventBusConnection = StateField.define({
  create(state) {
    const text = state.doc.toString();
    bus.emit("editor/change", { text, doc: state.doc });
    return buildAnalysis(text, state.doc);
  },
  update(value, tr) {
    if (!tr.docChanged) return value;
    const text = tr.state.doc.toString();
    bus.emit("editor/change", { text, doc: state.doc });
    return buildAnalysis(text, tr.state.doc);
  },
  provide: (f) => EditorView.decorations.from(f, (v) => v.deco),
});

function buildAnalysis(text: string, doc: Text) {
  save(text);
  const tokens = tokenize(text);
  const { log, ansi_log, machine } = compile(text);

  bus.emit("compiled", { log, ansi_log, machine });

  const marks = [];
  const docLen = doc.length;

  for (const tok of tokens) {
    const start = Math.max(0, Math.min(docLen, tok.start));
    const end = Math.max(start, Math.min(docLen, tok.end));
    let tc = tokenClass(tok.kind);
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
      if (end > start) {
        marks.push(Decoration.mark({ class: cls }).range(start, end));
      }
    }
  }

  const addDeco = (kind: HighlightKind, highlight: Span, location?: Span) => {
    if(!location) location = highlight;
    marks.push(Decoration.mark({attributes: {"highlight-kind": kind, "highlight-span": highlight_span_attr(highlight)}}).range(location[0], location[1]));
  };

  for (const transitions of machine?.transitions ?? []){
    for(const transition of transitions[1]){
      addDeco("focus", transition.function);
      addDeco("warning", transition.transition);
    }
  }

  for (const state of machine?.states.values() ?? []){
      addDeco("success", state.definition);
  }

  for (const [state, info] of machine?.final_states?.entries() ?? []){
    try{
      addDeco("success", machine?.states.get(state)!.definition!, info.definition);
    }catch(e){}
  }

  const deco = Decoration.set(marks, true);
  return { tokens, log, ansi_log, deco };
}

const tokenClass = (t: string) => ({
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

function severityClass(sev: string) {
  const s = (sev || "error").toLowerCase();
  if (s === "warning") return "cm-diag-warning";
  if (s === "info") return "cm-diag-info";
  return "cm-diag-error";
}
function sevRank(sev: string) {
  if (sev === "error") return 3;
  if (sev === "warning") return 2;
  return 1;
}

// ===================== Hover tooltip (uses cached diags) =====================
const diagHover = hoverTooltip((view, pos) => {
  const { log } = view.state.field(eventBusConnection);
  const hits = log.filter((d) =>
    d.start !== undefined && d.end !== undefined && pos >= d.start &&
    pos <= d.end
  );
  if (hits.length === 0) return null;

  const top = hits.reduce(
    (a, b) => (sevRank(b.level) > sevRank(a.level) ? b : a),
    hits[0],
  );

  return {
    pos,
    end: pos,
    above: true,
    create() {
      const dom = document.createElement("div");
      dom.className = "cm-tooltip cm-tooltip-hover";

      const title = document.createElement("div");
      title.className = `tipTitle ${top.level}`;
      title.textContent = hits.length === 1
        ? top.level.toUpperCase()
        : `${top.level.toUpperCase()} (${hits.length})`;

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

function save(text: string) {
  globalThis.localStorage.save = text;
}

function getSaved(): string | undefined {
  return globalThis.localStorage.save;
}

function defaultText(): string {
  return Share.sharedText() ?? getSaved() ?? examples[0].machine;
}

const state = EditorState.create({
  doc: defaultText(),
  extensions: [
    lineNumbers(),
    highlightActiveLineGutter(),
    history(),
    indentOnInput(),
    bracketMatching(),
    highlightActiveLine(),
    closeBrackets(),
    keymap.of([...defaultKeymap, ...historyKeymap]),

    eventBusConnection,
    diagHover,

    EditorView.lineWrapping,
  ],
});

const editor = new EditorView({
  state,
  parent: document.getElementById("editor")!,
});

bus.on(
  "begin",
  (_) => bus.emit("controls/editor/set_text", defaultText()),
);

bus.on("controls/editor/set_text", text => {
  editor.dispatch({
    changes: { from: 0, to: editor.state.doc.length, insert: text },
  });
});

bus.on("example/selected", example => {
  bus.emit("controls/editor/set_text", example.machine);
});
