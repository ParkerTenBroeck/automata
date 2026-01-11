import { network, updateVisualization } from "./visualizer.ts";

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

export type SimStepResult = "pending" | "accept" | "reject";

export class FaState {
  readonly state: State;

  readonly position: number;
  readonly input: string;
  readonly accepted: boolean = false;
  private repr!: string;

  constructor(state: State, position: number, input: string){
    this.state=state;
    this.position=position;
    this.input = input;
  }

  toString(): string{
    if(!this.repr) this.repr = this.state + " " + this.position;
    return this.repr
  }
};

export class FaSim {

  current_states: Map<string, FaState[]> = new Map();
  accepted: FaState[] = []
  
  step(): SimStepResult {
    return "pending";
  }
}

export class PdaState {
  readonly state: State;
  readonly stack: Symbol[];

  readonly position: number;
  readonly input: string;
  readonly accepted: boolean = false
  private repr!: string;

  constructor(state: State, stack: Symbol[], position: number, input: string){
    this.state=state;
    this.stack=stack;
    this.position=position;
    this.input = input;
  }

  toString(): string{
    if(!this.repr) this.repr = this.state + " [" + this.stack + "]" + " " + this.position;
    return this.repr
  }
};

export class PdaSim {
  machine: Pda;
  paths: PdaState[];
  input: string;

  current_states: Map<string, PdaState[]> = new Map();
  accepted: PdaState[] = []

  constructor(machine: Pda, input: string) {
    this.machine = machine;
    this.paths = [new PdaState(machine.initial_state, [machine.initial_stack], 0, input)];
    this.current_states.set(machine.initial_state, [this.paths[0]])
    this.input = input;
  }

  step(): SimStepResult {
    if (this.paths.length == 0) return "reject";
    if (this.accepted.length != 0) return "accept";

    const paths: PdaState[] = [];
    this.current_states.clear();

    const push = (state: PdaState) => {
      paths.push(state);
      if (!this.current_states.has(state.state)) this.current_states.set(state.state, []);
      this.current_states.get(state.state)?.push(state);

      if (
        state.position == this.input.length && this.machine.final_states &&
        this.machine.final_states.has(state.state)
       ||
        state.position == this.input.length && !this.machine.final_states &&
        state.stack.length == 1 && state.stack[0] == this.machine.initial_stack
      ) {
        
        // @ts-expect-error sillllyyyy
        state.accepted = true
        this.accepted.push(state);
      }
    };

    for (const path of this.paths) {

      const stack = path.stack.pop()!;
      const letter_map = this.machine.transitions_components.get(path.state)
        ?.get(stack);
      if (!letter_map) continue;

      for (const to of letter_map.get(null) ?? []) {
        push(new PdaState(to.state, path.stack.concat(to.stack), path.position, this.input));
      }

      if (path.position >= this.input.length) continue;

      const char = this.input.charAt(path.position);

      for (const to of letter_map.get(char) ?? []) {
        push(new PdaState(to.state, path.stack.concat(to.stack), path.position+1, this.input));
      }
    }
    this.paths = paths;


    if (this.paths.length == 0) return "reject";
    if (this.accepted.length != 0) return "accept";
    return "pending"
  }
}

export class TmState{
  readonly state: State;
  readonly tape: Symbol[];

  readonly position: number;
  readonly input: string;
  readonly accepted: boolean = false
  private repr!: string;

    constructor(state: State, tape: Symbol[], position: number, input: string){
    this.state=state;
    this.tape = tape;
    this.position=position;
    this.input = input;
  }

  toString(): string{
    if(!this.repr) this.repr = this.state + " " + this.position;
    return this.repr
  }

}

export class TmSim {
  current_states: Map<string, TmState[]> = new Map();
  accepted: TmState[] = []
  
  step(): SimStepResult {
    return "pending"
  }
} 

export type Sim = FaSim | PdaSim | TmSim | null 
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
  network.redraw()
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
  network.redraw()
}

export function resetSimulation(): void {
  switch (automaton.type) {
    case "fa":
      break;
    case "pda":
      setSimulation(new PdaSim(automaton as Pda, "aabbaabbaa"));
      break;
    case "tm":
      break;
  }
}
