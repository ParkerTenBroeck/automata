export type Machine = Fa | Pda | Tm;

export function parse_machine_from_json(json: string): Machine {
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
