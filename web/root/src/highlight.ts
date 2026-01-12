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
    count: number;
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

bus.on("automata/update", _ => {
    bus.emit("highlight/all/remove", undefined);
})

bus.on("highlight/one/add", (highlight) => {
    const key = asKey(highlight);
    if (current.has(key)) {
        current.get(key)!.count += 1;
    } else {
        current.set(key, { count: 1, ...highlight });
        bus.emit("highlight/update", undefined);
    }
});
bus.on("highlight/one/remove", (highlight) => {
    const key = asKey(highlight);
    if (current.has(key)) {
        const value = current.get(key)!
        value.count -= 1;
        if (value.count === 0) {
            current.delete(key);
            bus.emit("highlight/update", undefined);
        }
    }
});
bus.on("highlight/all/remove", (_) => {
    if (current.size !== 0) {
        current.clear();
        bus.emit("highlight/update", undefined);
    }
});