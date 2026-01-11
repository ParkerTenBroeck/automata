import { bus } from "./bus.ts";
import {
  Fa,
  Machine,
  parse_machine_from_json,
  Pda,
  State,
  Symbol,
  Tm,
} from "./automata.ts";

export type SimStepResult = "pending" | "accept" | "reject";
export type Sim = FaSim | PdaSim | TmSim;
let simulation: Sim | null = null;
let automaton: Machine = {
  type: "fa",
  alphabet: new Map(),
  final_states: new Map(),
  initial_state: "",
  states: new Map(),
  transitions: new Map(),
  transitions_components: new Map(),
  edges: new Map(),
};

bus.on("compiled", ({ machine }) => {
  if (machine) {
    try {
      bus.emit("controls/clear_simulation", undefined);
      automaton = parse_machine_from_json(machine);
      bus.emit("automata/update", { automaton });
    } catch (e) {
      console.log(e);
    }
  }
});
bus.on("controls/clear_simulation", (_) => {
  simulation = null;
  bus.emit("automata/sim/update", { simulation: null });
});
bus.on("controls/step_simulation", (_) => {
  if (simulation) {
    bus.emit("automata/sim/before_step", { simulation });
    bus.emit("automata/sim/after_step", {
      result: simulation.step(),
      simulation: simulation,
    });
  }
});
const machineInput = document.getElementById("machineInput") as HTMLInputElement;
machineInput.addEventListener("input", () => bus.emit("automata/sim/update", {simulation: null}));
machineInput.addEventListener("keydown", (e) => {
  if (e.key === "Enter") {
    bus.emit("controls/reload_simulation", undefined)
  }
});
bus.on("controls/reload_simulation", (_) => {
  const input = machineInput.value;
  switch (automaton.type) {
    case "fa":
      simulation = new FaSim(automaton as Fa, input);
      break;
    case "pda":
      simulation = new PdaSim(automaton as Pda, input);
      break;
    case "tm":
      simulation = new TmSim(automaton as Tm, input);
      break;
  }
  bus.emit("automata/sim/update", { simulation });
});
const simulationStatus = document.getElementById("simulationStatus") as HTMLInputElement;
bus.on("automata/sim/update", ({simulation}) => {
  if (!simulation){
    simulationStatus.innerText = "N/A"
    simulationStatus.style.color = "var(--fg-2)";
  }else{
    simulationStatus.innerText = "Pending"
    simulationStatus.style.color = "var(--warning)";
  }
});
bus.on("automata/sim/after_step", ({result}) => {
  if (result === "pending"){
    simulationStatus.innerText = "Pending"
    simulationStatus.style.color = "var(--warning)";
  }else if (result==="accept"){
    simulationStatus.innerText = "Accepted"
    simulationStatus.style.color = "var(--success)";
  }else if (result==="reject"){
    simulationStatus.innerText = "Rejected"
    simulationStatus.style.color = "var(--error)";
  }
});

export class FaState {
  readonly state: State;

  readonly position: number;
  readonly input: string;
  readonly accepted: boolean = false;
  private repr!: string;

  constructor(state: State, position: number, input: string) {
    this.state = state;
    this.position = position;
    this.input = input;
  }

  toString(): string {
    if (!this.repr) {
      this.repr = this.state + +" >" + this.input.substring(this.position);
    }
    return this.repr;
  }
}

export class FaSim {
  machine: Fa;
  paths: FaState[];
  input: string;

  current_states: Map<string, FaState[]> = new Map();
  accepted: FaState[] = [];

  constructor(machine: Fa, input: string) {
    this.machine = machine;
    this.paths = [new FaState(machine.initial_state, 0, input)];
    this.current_states.set(machine.initial_state, [this.paths[0]]);
    this.input = input;
  }

  step(): SimStepResult {
    if (this.paths.length == 0) return "reject";
    if (this.accepted.length != 0) return "accept";

    const paths: FaState[] = [];
    this.current_states.clear();

    const push = (state: FaState) => {
      paths.push(state);
      if (!this.current_states.has(state.state)) {
        this.current_states.set(state.state, []);
      }
      this.current_states.get(state.state)?.push(state);

      if (
        state.position == this.input.length &&
        this.machine.final_states.has(state.state)
      ) {
        // @ts-expect-error sillllyyyy
        state.accepted = true;
        this.accepted.push(state);
      }
    };

    for (const path of this.paths) {
      const letter_map = this.machine.transitions_components.get(path.state)!;
      if (!letter_map) continue;

      for (const to of letter_map.get(null) ?? []) {
        push(new FaState(to.state, path.position, this.input));
      }

      if (path.position >= this.input.length) continue;

      const char = this.input.charAt(path.position);

      for (const to of letter_map.get(char) ?? []) {
        push(new FaState(to.state, path.position + 1, this.input));
      }
    }
    this.paths = paths;

    if (this.paths.length == 0) return "reject";
    if (this.accepted.length != 0) return "accept";
    return "pending";
  }
}

export class PdaState {
  readonly state: State;
  readonly stack: Symbol[];

  readonly position: number;
  readonly input: string;
  readonly accepted: boolean = false;
  private repr!: string;

  constructor(state: State, stack: Symbol[], position: number, input: string) {
    this.state = state;
    this.stack = stack;
    this.position = position;
    this.input = input;
  }

  toString(): string {
    if (!this.repr) {
      this.repr = this.state + " [" + this.stack + "]" + " >" +
        this.input.substring(this.position);
    }
    return this.repr;
  }
}

export class PdaSim {
  machine: Pda;
  paths: PdaState[];
  input: string;

  current_states: Map<string, PdaState[]> = new Map();
  accepted: PdaState[] = [];

  constructor(machine: Pda, input: string) {
    this.machine = machine;
    this.paths = [
      new PdaState(machine.initial_state, [machine.initial_stack], 0, input),
    ];
    this.current_states.set(machine.initial_state, [this.paths[0]]);
    this.input = input;
  }

  step(): SimStepResult {
    if (this.paths.length == 0) return "reject";
    if (this.accepted.length != 0) return "accept";

    const paths: PdaState[] = [];
    this.current_states.clear();

    const push = (state: PdaState) => {
      paths.push(state);
      if (!this.current_states.has(state.state)) {
        this.current_states.set(state.state, []);
      }
      this.current_states.get(state.state)?.push(state);

      if (
        state.position == this.input.length && this.machine.final_states &&
          this.machine.final_states.has(state.state) ||
        state.position == this.input.length && !this.machine.final_states &&
          state.stack.length == 1 &&
          state.stack[0] == this.machine.initial_stack
      ) {
        // @ts-expect-error sillllyyyy
        state.accepted = true;
        this.accepted.push(state);
      }
    };

    for (const path of this.paths) {
      const stack = path.stack.pop()!;
      const letter_map = this.machine.transitions_components.get(path.state)
        ?.get(stack);
      if (!letter_map) continue;

      for (const to of letter_map.get(null) ?? []) {
        push(
          new PdaState(
            to.state,
            path.stack.concat(to.stack),
            path.position,
            this.input,
          ),
        );
      }

      if (path.position >= this.input.length) continue;

      const char = this.input.charAt(path.position);

      for (const to of letter_map.get(char) ?? []) {
        push(
          new PdaState(
            to.state,
            path.stack.concat(to.stack),
            path.position + 1,
            this.input,
          ),
        );
      }
    }
    this.paths = paths;

    if (this.paths.length == 0) return "reject";
    if (this.accepted.length != 0) return "accept";
    return "pending";
  }
}

export class TmState {
  readonly state: State;
  readonly tape: Symbol[];

  readonly position: number;
  readonly input: string;
  readonly accepted: boolean = false;
  private repr!: string;

  constructor(state: State, tape: Symbol[], position: number, input: string) {
    this.state = state;
    this.tape = tape;
    this.position = position;
    this.input = input;
  }

  toString(): string {
    if (!this.repr) this.repr = this.state + " " + this.position;
    return this.repr;
  }
}

export class TmSim {
  machine: Tm;
  input: string;
  current_states: Map<string, TmState[]> = new Map();
  accepted: TmState[] = [];

  constructor(machine: Tm, input: string) {
    this.machine = machine;
    this.input = input;
  }

  step(): SimStepResult {
    return "pending";
  }
}
