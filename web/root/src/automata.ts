import { updateVisualization } from "./visualizer.ts";

export type Machine = Fa | Pda | Tm;

export function machine_from_json(json: string): Machine {
  const machine: Machine = JSON.parse(json);
  machine.states = new Map(Object.entries(machine.states));

  if (machine.alphabet) {
    machine.alphabet = new Map(Object.entries(machine.alphabet));
  }
  if (machine.final_states) {
    machine.final_states = new Map(Object.entries(machine.final_states));
  }

  // deno-lint-ignore no-explicit-any
  const transitions = machine.transitions as any as [any, any];
  machine.transitions = new Map();
  for (const [key, value] of transitions) {
    machine.transitions.set(key, value);
  }
  machine.edges = new Map();

  machine.transitions_components = new Map();
  switch (machine.type) {
    case "fa":
      {
        for (const [from, tos] of machine.transitions) {
          for (const to of tos) {
            const layer_0 = machine.transitions_components;
            if (!layer_0.has(from.state)) layer_0.set(from.state, new Map());
            const layer_1 = machine.transitions_components.get(from.state)!;
            if (!layer_1.has(from.letter)) layer_1.set(from.letter, []);
            const layer_2 = layer_1.get(from.letter)!;
            layer_2.push(to);

            const edge = from.state + "#" + to.state;
            if (!machine.edges.has(edge)) machine.edges.set(edge, []);
            machine.edges.get(edge)?.push({
              repr: from.letter ? from.letter : "ε",
              function: to.function,
              transition: to.transition,
            });
          }
        }
      }
      break;
    case "pda":
      {
        machine.symbols = new Map(Object.entries(machine.symbols));
        for (const [from, tos] of machine.transitions) {
          for (const to of tos) {
            const layer_0 = machine.transitions_components;
            if (!layer_0.has(from.state)) layer_0.set(from.state, new Map());
            const layer_1 = machine.transitions_components.get(from.state)!;
            if (!layer_1.has(from.symbol)) layer_1.set(from.symbol, new Map());
            const layer_2 = layer_1.get(from.symbol)!;
            if (!layer_2.has(from.letter)) layer_2.set(from.letter, []);
            const layer_3 = layer_2.get(from.letter)!;
            layer_3.push(to);

            const edge = from.state + "#" + to.state;
            if (!machine.edges.has(edge)) machine.edges.set(edge, []);
            machine.edges.get(edge)?.push({
              repr: (from.letter ? from.letter : "ε") + "," + from.symbol +
                "->[" + to.stack + "]",
              function: to.function,
              transition: to.transition,
            });
          }
        }
      }
      break;
    case "tm":
      {
        machine.symbols = new Map(Object.entries(machine.symbols));
        for (const [from, tos] of machine.transitions) {
          for (const to of tos) {
            const layer_0 = machine.transitions_components;
            if (!layer_0.has(from.state)) layer_0.set(from.state, new Map());
            const layer_1 = machine.transitions_components.get(from.state)!;
            if (!layer_1.has(from.symbol)) layer_1.set(from.symbol, []);
            const layer_2 = layer_1.get(from.symbol)!;
            layer_2.push(to);

            const edge = from.state + "#" + to.state;
            if (!machine.edges.has(edge)) machine.edges.set(edge, []);
            machine.edges.get(edge)?.push({
              repr: from.symbol + "->" + to.symbol + "," + to.direction,
              function: to.function,
              transition: to.transition,
            });
          }
        }
      }
      break;
  }
  return machine;
}

export type State = string;
export type Symbol = string;
export type Letter = string;

export type Span = [number, number];

export type StateInfo = { definition: Span };
export type LetterInfo = { definition: Span };
export type SymbolInfo = { definition: Span };

export type FaTransFrom = {
  state: State;
  letter: Letter | null;
};

export type FaTransTo = {
  state: State;

  transition: Span;
  function: Span;
};

export type Edge = {
  repr: string;
  function: Span;
  transition: Span;
};

export type Fa = {
  type: "fa";

  initial_state: State;
  states: Map<State, StateInfo>;
  alphabet: Map<Letter, LetterInfo>;
  final_states: Map<State, StateInfo>;

  transitions: Map<FaTransFrom, FaTransTo[]>;
  transitions_components: Map<State, Map<Letter | null, FaTransTo[]>>;

  edges: Map<string, Edge[]>;
};

export type PdaTransFrom = {
  state: State;
  letter: Letter | null;
  symbol: Symbol;
};

export type PdaTransTo = {
  state: State;
  stack: Symbol[];

  transition: Span;
  function: Span;
};

export type Pda = {
  type: "pda";

  initial_state: State;
  initial_stack: Symbol;
  states: Map<State, StateInfo>;
  symbols: Map<Symbol, SymbolInfo>;
  alphabet: Map<Letter, LetterInfo>;
  final_states: Map<State, StateInfo> | null;

  transitions: Map<PdaTransFrom, PdaTransTo[]>;
  transitions_components: Map<
    State,
    Map<Symbol, Map<Letter | null, PdaTransTo[]>>
  >;

  edges: Map<string, Edge[]>;
};

export type TmTransFrom = {
  state: State;
  symbol: Symbol;
};

export type TmTransTo = {
  state: State;
  symbol: Symbol;
  direction: "L" | "R" | "N";

  transition: Span;
  function: Span;
};

export type Tm = {
  type: "tm";

  initial_state: State;
  initial_tape: Symbol;
  states: Map<State, StateInfo>;
  symbols: Map<Symbol, SymbolInfo>;
  alphabet: Map<Letter, LetterInfo>;
  final_states: Map<State, StateInfo>;

  transitions: Map<TmTransFrom, TmTransTo[]>;
  transitions_components: Map<State, Map<Symbol, TmTransTo[]>>;

  edges: Map<string, Edge[]>;
};

export type FaState = {
  state: State;
  position: number;
};

export class FaSim {
  step(): string {
    return "";
  }
}

export type PdaState = {
  state: State;
  stack: Symbol[];
  position: number;
};

export class PdaSim {
  machine: Pda;
  paths: PdaState[];
  input: string;

  constructor(machine: Pda, input: string) {
    this.machine = machine;
    this.paths = [{
      state: machine.initial_state,
      stack: [machine.initial_stack],
      position: 0,
    }];
    this.input = input;
  }

  step(): string {
    const paths = [];
    console.log(this.paths);
    for (const path of this.paths) {
      if (
        path.position == this.input.length && this.machine.final_states &&
        this.machine.final_states.has(path.state)
      ) return "accept";
      if (
        path.position == this.input.length && !this.machine.final_states &&
        path.stack.length == 1 && path.stack[0] == this.machine.initial_stack
      ) return "accept";

      const stack = path.stack.pop()!;
      const letter_map = this.machine.transitions_components.get(path.state)
        ?.get(stack);
      if (!letter_map) continue;

      for (const to of letter_map.get(null) ?? []) {
        paths.push({
          state: to.state,
          position: path.position,
          stack: path.stack.concat(to.stack),
        });
      }

      if (path.position >= this.input.length) continue;

      const char = this.input.charAt(path.position);

      for (const to of letter_map.get(char) ?? []) {
        paths.push({
          state: to.state,
          position: path.position + 1,
          stack: path.stack.concat(to.stack),
        });
      }
    }
    this.paths = paths;
    return paths.length == 0 ? "reject" : "pending";
  }
}

export type Sim = FaSim | PdaSim | null 
export let sim: Sim = null;

export let automaton: Machine = {
  type: "fa",
  alphabet: new Map(),
  final_states: new Map(),
  initial_state: "",
  states: new Map(),
  transitions: new Map(),
  transitions_components: new Map(),
  edges: new Map(),
};

export function clearSimulation(){
  setSimulation(null);
}

export function setSimulation(sim_: Sim){
  sim = sim_;
}

export function setAutomaton(auto: Machine) {
  automaton = auto;
  sim = null;
  updateVisualization()
}

export function clearAutomaton() {
  setAutomaton({
    type: "fa",
    alphabet: new Map(),
    final_states: new Map(),
    initial_state: "",
    states: new Map(),
    transitions: new Map(),
    transitions_components: new Map(),
    edges: new Map(),
  });
}

export function stepSimulation(): void {
  if (sim) {
    console.log(sim.step());
  }
}

export function resetSimulation(): void {
  switch (automaton.type) {
    case "fa":
      break;
    case "pda":
      setSimulation(new PdaSim(automaton as Pda, "aabb"));
      break;
    case "tm":
      break;
  }
}
