import { bus } from "./bus.ts";
import type {
  Machine,
  Fa,
  Pda,
  Tm,
} from "./automata.ts";
import {parse_machine_from_json} from "./automata.ts";

import { FaSim } from "./simulation/fa.ts";
export { FaSim } from "./simulation/fa.ts";

import { PdaSim } from "./simulation/pda.ts";
export { PdaSim } from "./simulation/pda.ts";

import { TmSim } from "./simulation/tm.ts";
export { TmSim } from "./simulation/tm.ts";

export type SimStepResult = "pending" | "accept" | "reject";
export type Sim = FaSim | PdaSim | TmSim;

export let simulation: Sim | null = null;
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

bus.on("compiled", ({ machine }) => {
  if (machine) {
    try {
      bus.emit("controls/sim/clear", undefined);
      automaton = parse_machine_from_json(machine);
      bus.emit("automata/update", automaton);
    } catch (e) {
      console.log(e);
    }
  }
});
bus.on("controls/sim/clear", (_) => {
  simulation = null;
  bus.emit("automata/sim/update", null);
});
bus.on("controls/sim/step", (_) => {
  if (simulation) {
    bus.emit("automata/sim/before_step", { simulation });
    bus.emit("automata/sim/after_step", {
      result: simulation.step(),
      simulation: simulation,
    });
  }
});
const machineInput = document.getElementById("machineInput") as HTMLInputElement;
machineInput.addEventListener("input", () => bus.emit("controls/sim/clear", undefined));
machineInput.addEventListener("keydown", (e) => {
  if (e.key === "Enter") {
    bus.emit("controls/sim/reload", undefined)
  }
});
bus.on("controls/sim/reload", (_) => {
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
  bus.emit("automata/sim/update", simulation);
});
const simulationStatus = document.getElementById("simulationStatus") as HTMLInputElement;
bus.on("automata/sim/update", simulation => {
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

