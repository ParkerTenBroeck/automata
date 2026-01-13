import type {
    Fa,
    FaTransTo,
    State,
} from "../automata.ts";
import { SimStepResult } from "../simulation.ts";

export type FaState = {
  readonly state: State;
  readonly position: number;

  readonly accepted: boolean;
  readonly repr: string;

  readonly path: readonly FaTransTo[];
};

type Initializer<T> = { -readonly [P in keyof T]?: T[P] | undefined };

export class FaSim {
  readonly machine: Fa;
  readonly input: string;

  paths: FaState[] = [];
  current_states: Map<string, FaState[]> = new Map();
  accepted: FaState[] = [];
  rejected: FaState[] = [];

  constructor(machine: Fa, input: string) {
    this.machine = machine;
    this.input = input;
    this.initial();
  }

  private accept(state: Initializer<FaState>): boolean {
    const pos = state.position ?? 0;
    const st = state.state!;
    return pos === this.input.length && this.machine.final_states.has(st);
  }

  private init_state(state: Initializer<FaState>) {
    state.position ??= 0;

    state.accepted = this.accept(state);
    state.repr = state.state + " >" + this.input.substring(state.position);

    const frozen = state as FaState;

    if (frozen.accepted) this.accepted.push(frozen);
    this.paths.push(frozen);

    if (!this.current_states.has(frozen.state)) {
      this.current_states.set(frozen.state, []);
    }
    this.current_states.get(frozen.state)!.push(frozen);
  }

  private initial() {
    const state: Initializer<FaState> = {
      state: this.machine.initial_state,
      position: 0,
      path: [],
    };

    this.init_state(state);
  }

  private transition(from: FaState, to: FaTransTo, consume: boolean) {
    const state: Initializer<FaState> = {
      state: to.state,
      position: from.position + (consume ? 1 : 0),
      path: from.path.concat([to]),
    };

    this.init_state(state);
  }

  step(): SimStepResult {
    if (this.accepted.length !== 0) return "accept";
    if (this.paths.length === 0) return "reject";

    const paths = this.paths;
    this.paths = [];
    this.current_states.clear();

    for (const from of paths) {
      const letterMap = this.machine.transitions_components.get(from.state);

      if (!letterMap) {
        this.rejected.push(from);
        continue;
      }

      // epsilon transitions
      const eps = letterMap.get(null) ?? [];
      for (const to of eps) this.transition(from, to, false);

      // consuming transitions
      if (from.position >= this.input.length) {
        if (eps.length === 0) this.rejected.push(from);
        continue;
      }

      const ch = this.input.charAt(from.position);
      const trs = letterMap.get(ch) ?? [];
      for (const to of trs) this.transition(from, to, true);

      if (eps.length === 0 && trs.length === 0) {
        this.rejected.push(from);
      }
    }

    if (this.accepted.length !== 0) return "accept";
    if (this.paths.length === 0) return "reject";
    return "pending";
  }
}
