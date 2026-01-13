import type {
    Pda,
    PdaTransTo,
    State,
    Symbol
} from "../automata.ts";
import { SimStepResult } from "../simulation.ts";
import { EPSILON } from "../constants.ts";


export type Step = PdaTransTo & {from_state: State, from_letter: string, from_stack: Symbol}
export type PdaState = {
    readonly state: State;
    readonly stack: Symbol[];
    readonly position: number;

    readonly accepted: boolean;
    readonly repr: string;

    readonly path: readonly Step[];
};

type Initializer<T> = { -readonly [P in keyof T]?: T[P] | undefined };

export class PdaSim {
    readonly machine: Pda;
    readonly input: string;

    paths: PdaState[] = [];
    current_states: Map<string, PdaState[]> = new Map();
    accepted: PdaState[] = [];
    rejected: PdaState[] = [];

    constructor(machine: Pda, input: string) {
        this.machine = machine;
        this.input = input;
        this.initial();
    }

    private accept(state: Initializer<PdaState>): boolean {
        const pos = state.position ?? 0;
        const st = state.state!;
        const stack = state.stack ?? [];

        //accept by final state
        if (pos === this.input.length && this.machine.final_states && this.machine.final_states.has(st)) {
            return true;
        }
        //accept by empty stack
        if (pos === this.input.length && !this.machine.final_states && stack.length === 1 && stack[0] === this.machine.initial_stack) {
            return true;
        }

        return false;
    }

    private init_state(state: Initializer<PdaState>) {
        state.stack ??= [this.machine.initial_stack];
        state.position ??= 0;

        state.accepted = this.accept(state);
        state.repr = state.state + " [" + state.stack.join(",") + "] >" + this.input.substring(state.position);

        const frozen = state as PdaState;

        if (frozen.accepted) this.accepted.push(frozen);
        this.paths.push(frozen);

        if (!this.current_states.has(frozen.state)) {
            this.current_states.set(frozen.state, []);
        }
        this.current_states.get(frozen.state)!.push(frozen);
    }

    private initial() {
        const state: Initializer<PdaState> = {
            state: this.machine.initial_state,
            stack: [this.machine.initial_stack],
            position: 0,
            path: [],
        };

        this.init_state(state);
    }

    private transition(from: PdaState, to: PdaTransTo, letter: string|undefined) {
        const stackCopy = from.stack.slice(0, from.stack.length - 1); // pop off top
        const nextStack = stackCopy.concat(to.stack);
        if (nextStack.length == 0) {
            this.rejected.push(from)
            return;
        }

        const state: Initializer<PdaState> = {
            state: to.state,
            stack: nextStack,
            position: from.position + (letter ? 1 : 0),
            path: from.path.concat([{from_state: from.state, from_letter: letter??EPSILON, from_stack: from.stack[from.stack.length-1], ...to}]),
        };

        this.init_state(state);
    }

    step(): SimStepResult {

        const paths = this.paths;
        this.paths = [];
        this.current_states.clear();

        for (const from of paths) {
            const top = from.stack[from.stack.length - 1];

            const letterMap = this.machine.transitions_components.get(from.state)?.get(top);
            if (!letterMap) {
                this.rejected.push(from);
                continue;
            }

            // epsilon transitions
            const epsilon_transitions = letterMap.get(null) ?? [];
            for (const to of epsilon_transitions) {
                this.transition(from, to, undefined);
            }

            if (from.position >= this.input.length) {
                if (epsilon_transitions.length == 0){
                    this.rejected.push(from);
                }
                continue;
            }
            // consuming transitions
            const ch = this.input.charAt(from.position);

            const transitions = letterMap.get(ch) ?? [];
            for (const to of transitions) {
                this.transition(from, to, ch);
            }
            if (epsilon_transitions.length == 0 && transitions.length == 0){
                this.rejected.push(from);
            }
        }
        
        return this.status();
    }

    status(): SimStepResult {
        if (this.accepted.length !== 0) return "accept";
        if (this.paths.length === 0) return "reject";
        return "pending";
    }
}