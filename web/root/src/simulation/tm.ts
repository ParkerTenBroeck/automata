import type {
  State,
  Symbol,
  Tm,
  TmTransTo
} from "../automata.ts";
import { SimStepResult } from "../simulation.ts";


export type TmState = {
  readonly state: State;
  readonly tape: Symbol[];
  readonly head: number;

  readonly accepted: boolean;
  readonly repr: string;

  readonly path: readonly TmTransTo[];
}


type Initializer<T> = { -readonly [P in keyof T]?: T[P] | undefined };

export class TmSim {
  readonly machine: Tm;
  paths: TmState[] = [];
  readonly input: string;

  current_states: Map<string, TmState[]> = new Map();
  accepted: TmState[] = [];
  rejected: TmState[] = [];

  constructor(machine: Tm, input: string) {
    this.machine = machine;
    this.input = input;
    this.initial();
  }

  private init_state(state: Initializer<TmState>) {
    state.repr = state.state + " [ " + this.machine.blank_symbol + " " + state.tape!.map((s, i, _) => i == state.head ? `[${s}]` : s).join(" ") + " " + this.machine.blank_symbol + " ]";


    const frozen = state as TmState;
    if (frozen.accepted) this.accepted.push(frozen);
    this.paths.push(frozen);
    if (!this.current_states.has(frozen.state)) {
      this.current_states.set(frozen.state, []);
    }
    this.current_states.get(frozen.state)!.push(frozen);
  }

  private initial() {
    const state: Initializer<TmState> = {
      state: this.machine.initial_state,
      accepted: this.machine.final_states.has(this.machine.initial_state),
      tape: this.input.split(''),
      head: 0,

      path: [],
    };

    if (state.tape!.length == 0) state.tape!.push(this.machine.blank_symbol)

    this.init_state(state);
  }

  private transition(from: TmState, to: TmTransTo) {
    const state: Initializer<TmState> = {
      state: to.state,
      accepted: this.machine.final_states.has(to.state),

      path: from.path.concat([to]),
    };

    switch (to.direction) {
      case "_":
        state.tape = from.tape.slice();
        state.tape![from.head] = to.symbol;
        state.head = from.head;
        break;
      case "<":
        if (from.head == 0) {
          state.tape = from.tape.splice(0, 0, to.symbol);
          state.head = 0;
        } else {
          state.tape = from.tape.slice();
          state.tape![from.head] = to.symbol;
          state.head = from.head - 1;
        }
        break;
      case ">":
        state.head = from.head + 1;
        state.tape = from.tape.slice();
        state.tape![from.head] = to.symbol;
        if (state.head == from.tape.length) {
          state.tape!.push(this.machine.blank_symbol);
        }
        break;
    }

    this.init_state(state)
  }

  step(): SimStepResult {
    if (this.accepted.length != 0) return "accept";
    if (this.paths.length == 0) return "reject";

    const paths: TmState[] = this.paths;
    this.paths = [];
    this.current_states.clear();

    for (const from of paths) {
      const symbol = from.tape[from.head];
      const transitions = this.machine.transitions_components.get(from.state)?.get(symbol) ?? [];
      if (transitions.length == 0) {
        this.rejected.push(from);
        continue;
      }

      for (const to of transitions) {
        this.transition(from, to);
      }
    }

    if (this.accepted.length != 0) return "accept";
    if (this.paths.length == 0) return "reject";
    return "pending";
  }
}