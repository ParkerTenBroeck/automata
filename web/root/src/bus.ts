// deno-lint-ignore-file

import type { Machine } from "./automata.ts";
import type { Example } from "./examples.ts";
import type { Sim, SimStepResult } from "./simulation.ts";
import type wasm from "./wasm.ts";
import type { Text } from "npm:@codemirror/state";

type Unsubscribe = () => void;

export class EventBus<Events extends Record<string, any>> {
  private listeners = new Map<keyof Events, Set<(payload: any) => void>>();

  on<K extends keyof Events>(
    event: K,
    handler: (payload: Events[K]) => void,
  ): Unsubscribe {
    let set = this.listeners.get(event);
    if (!set) {
      set = new Set();
      this.listeners.set(event, set);
    }
    set.add(handler as any);

    return () => this.off(event, handler);
  }

  once<K extends keyof Events>(
    event: K,
    handler: (payload: Events[K]) => void,
  ): Unsubscribe {
    const off = this.on(event, (payload) => {
      off();
      handler(payload);
    });
    return off;
  }

  off<K extends keyof Events>(event: K, handler: (payload: Events[K]) => void) {
    const set = this.listeners.get(event);
    if (!set) return;
    set.delete(handler as any);
    if (set.size === 0) this.listeners.delete(event);
  }

  emit<K extends keyof Events>(event: K, payload: Events[K]) {
    const set = this.listeners.get(event);
    if (!set) return;

    // Copy to avoid issues if handlers subscribe/unsubscribe during emit
    for (const handler of Array.from(set)) {
      try {
        (handler as (p: Events[K]) => void)(payload);
      } catch (e) {
        console.log(e);
      }
    }
  }

  clear(event?: keyof Events) {
    if (event) this.listeners.delete(event);
    else this.listeners.clear();
  }
}

type AppEvents = {
  "begin": void;

  "editor/change": {text: string, doc: Text};
  "compiled": {log: wasm.CompileLog[], ansi_log: string, machine: string|undefined};

  "automata/sim/update": { simulation: Sim|null };
  "automata/sim/before_step": { simulation: Sim };
  "automata/sim/after_step": { simulation: Sim, result: SimStepResult };
  "automata/update": { automaton: Machine };

  "example/selected": {example: Example};

  "controls/editor/set_text": {text: string};
  
  "controls/vis/physics": {enabled: boolean};
  "controls/vis/reset_network": void;
  
  "controls/sim/step": void;
  "controls/sim/reload": void;
  "controls/sim/clear": void;

  "theme/update": void;
};

export const bus = new EventBus<AppEvents>();
