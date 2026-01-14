import type { Span } from "./automata.ts";
import { bus } from "./bus.ts";
import { automaton } from "./simulation.ts";


export type HighlightKind = "focus" | "error" | "warning" | "success";



export type Highlight = {
    span: Span,
    kind: HighlightKind,
}

type HighlightEntry = {
    span: Span,
    kind: HighlightKind,
}

export const current: Map<string, HighlightEntry> = new Map();


function asKey(highlight: Highlight): string {
    return `${highlight.span[0]}:${highlight.span[1]}:${highlight.kind}`
}

export function highlight_from_node_id(node_id: string) {
    const state = automaton.states.get(node_id);
    if (state) {
        bus.emit("highlight/one/add", { kind: "success", span: state.definition })
    }
}

export function dehighlight_from_node_id(node_id: string) {
    const state = automaton.states.get(node_id);
    if (state) {
        bus.emit("highlight/one/remove", { kind: "success", span: state.definition })
    }
}

export function highlight_from_edge_id(node_id: string) {
    for (const edge_value of automaton.edges.get(node_id)!) {
        bus.emit("highlight/one/add", { kind: "focus", span: edge_value.function })
        bus.emit("highlight/one/add", { kind: "warning", span: edge_value.transition })
    }
}

export function dehighlight_from_edge_id(node_id: string) {
    for (const edge_value of automaton.edges.get(node_id)!) {
        bus.emit("highlight/one/remove", { kind: "focus", span: edge_value.function })
        bus.emit("highlight/one/remove", { kind: "warning", span: edge_value.transition })
    }
}

function decoForKind(kind: HighlightKind): string {
    return `cm-highlight-${kind}`;
}

bus.on("highlight/one/add", (highlight) => {
    const key = asKey(highlight);
    if (!current.has(key)) {
        current.set(key, {...highlight });
        
        const cname = decoForKind(highlight.kind);
        const repr = `${highlight.span[0]}:${highlight.span[1]}`;
        globalThis.document.querySelectorAll(`[highlight-span="${repr}"]`).forEach(el => el.classList.add(cname))

        bus.emit("highlight/update", {repr, remove: false, ...highlight});
    }
});
bus.on("highlight/one/remove", (highlight) => {
    const key = asKey(highlight);
    if (current.delete(key)) {
        const cname = decoForKind(highlight.kind);
        const repr = `${highlight.span[0]}:${highlight.span[1]}`;
        globalThis.document.querySelectorAll(`[highlight-span="${repr}"]`).forEach(el => el.classList.remove(cname))

        bus.emit("highlight/update", {repr, remove: true, ...highlight});
    }
});


globalThis.document.addEventListener("mouseover", (e) => {
  if (!(e.target instanceof Element)) return;

  const target = e.target.closest("[highlight-span]");
  if (!target) return;

  const related = e.relatedTarget instanceof Element
    ? e.relatedTarget.closest("[highlight-span]")
    : null;

  // Mouse is still inside the same highlight span → ignore
  if (related === target) return;

  const kind = (target.getAttribute("highlight-kind") ?? "focus") as unknown as HighlightKind;
  const span = target.getAttribute("highlight-span")!.split(":").map(Number) as unknown as Span;
  
  bus.emit("highlight/one/add", {span, kind});
});

document.addEventListener("mouseout", (e) => {
  if (!(e.target instanceof Element)) return;

  const from = e.target.closest("[highlight-span]");
  const to = e.relatedTarget instanceof Element
    ? e.relatedTarget.closest("[highlight-span]")
    : null;

  if (!from || from === to) return;

  const kind = (from.getAttribute("highlight-kind") ?? "focus") as unknown as HighlightKind;
  const span = from.getAttribute("highlight-span")!.split(":").map(Number) as unknown as Span;

  bus.emit("highlight/one/remove", {span, kind});
});

export function highlightable(span: Span, text: string, kind?: HighlightKind): string{
  return `<span class = "cm-highlight" ${kind ? `highlight-kind="${kind}"`:""} highlight-span="${span[0]}:${span[1]}">${text}</span>`
}

export function highlight_span_attr(span: Span): string{
    return `${span[0]}:${span[1]}`   
}