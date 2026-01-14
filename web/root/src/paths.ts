import { bus } from "./bus.ts";
import { DELTA } from "./constants.ts";
import { highlightable } from "./highlight.ts";
import type { Sim } from "./simulation.ts";
import type { FaState } from "./simulation/fa.ts";
import type { PdaState } from "./simulation/pda.ts";
import type { TmState } from "./simulation/tm.ts";



function renderFaPath(state: FaState, index: number) {
  const details = document.createElement("details");
  details.className = "pathItem";

  const summary = document.createElement("summary");
  summary.className = "pathHeader";
  summary.innerHTML = `
    <span>${state.repr}</span>
    <span class="pathMeta">
      <span>steps: ${state.path.length}</span>
    </span>
  `;

  const steps = document.createElement("div");
  steps.className = "steps";

  for (let i = 0; i < state.path.length; i++) {
    const div = document.createElement("div");
    div.className = "stepLine";
    const step = state.path[i];
    
    div.innerHTML = `${i + 1}. ` 
      + highlightable(step.function, `${DELTA}(${step.from_state})`, "focus") 
      + " = " 
      + highlightable(step.transition, step.state, "warning");
    steps.appendChild(div);
  }

  details.appendChild(summary);
  details.appendChild(steps);
  return details;
}

function renderPdaPath(state: PdaState, index: number) {
  const details = document.createElement("details");
  details.className = "pathItem";

  const summary = document.createElement("summary");
  summary.className = "pathHeader";
  summary.innerHTML = `
    <span>${state.repr}</span>
    <span class="pathMeta">
      <span>steps: ${state.path.length}</span>
    </span>
  `;

  const steps = document.createElement("div");
  steps.className = "steps";

  for (let i = 0; i < state.path.length; i++) {
    const div = document.createElement("div");
    div.className = "stepLine";
    const step = state.path[i];

    div.innerHTML = `${i + 1}. ` 
      + highlightable(step.function, `${DELTA}(${step.from_state}, ${step.from_letter}, , ${step.from_stack})`, "focus") 
      + " = " 
      + highlightable(step.transition, `(${step.state}, [ ${step.stack.join(" ")} ])`, "warning");
    steps.appendChild(div);
  }

  details.appendChild(summary);
  details.appendChild(steps);
  return details;
}

function renderTmPath(state: TmState, index: number) {
  const details = document.createElement("details");
  details.className = "pathItem";

  const summary = document.createElement("summary");
  summary.className = "pathHeader";
  summary.innerHTML = `
    <span>${state.repr}</span>
    <span class="pathMeta">
      <span>steps: ${state.path.length}</span>
    </span>
  `;

  const steps = document.createElement("div");
  steps.className = "steps";

  for (let i = 0; i < state.path.length; i++) {
    const div = document.createElement("div");
    div.className = "stepLine";
    const step = state.path[i];
    div.setAttribute("highlight-span", "${}")

    div.innerHTML = `${i + 1}. ` 
      + highlightable(step.function, `${DELTA}(${step.from_state}, ${step.from_symbol})`, "focus") 
      + " = " 
      + highlightable(step.transition, `(${step.state}, ${step.symbol}, ${step.direction})`, "warning");
    console.log(div.innerHTML);
    steps.appendChild(div);
  }

  details.appendChild(summary);
  details.appendChild(steps);
  return details;
}

bus.on("automata/sim/after_step", ({simulation}) => {
    renderPaths(simulation)
})

bus.on("automata/sim/update", simulation => {
    if(simulation){
        renderPaths(simulation)
    }else{
        renderPaths(undefined) 
    }
})

export function renderPaths(sim: Sim | undefined) {
  const acceptedEl = document.getElementById("acceptedPaths")!;
  const runningEl = document.getElementById("runningPaths")!;
  const rejectedEl = document.getElementById("rejectedPaths")!;

  const acceptedCount = document.getElementById("acceptedCount")!;
  const runningCount = document.getElementById("runningCount")!;
  const rejectedCount = document.getElementById("rejectedCount")!;


  acceptedEl.innerHTML = "";
  runningEl.innerHTML = "";
  rejectedEl.innerHTML = "";

  acceptedCount.textContent = String(sim?.accepted.length ?? 0);
  runningCount.textContent = String(sim?.paths.length ?? 0);
  rejectedCount.textContent = String(sim?.rejected.length ?? 0);

  if(!sim)return;
  switch (sim.machine.type){
    case "fa": 
      sim.accepted.forEach((s, i) => acceptedEl.appendChild(renderFaPath(s as FaState, i)));
      sim.paths.forEach((s, i) => runningEl.appendChild(renderFaPath(s as FaState, i)));
      sim.rejected.forEach((s, i) => rejectedEl.appendChild(renderFaPath(s as FaState, i)));
    break;
    case "pda": 
      sim.accepted.forEach((s, i) => acceptedEl.appendChild(renderPdaPath(s as PdaState, i)));
      sim.paths.forEach((s, i) => runningEl.appendChild(renderPdaPath(s as PdaState, i)));
      sim.rejected.forEach((s, i) => rejectedEl.appendChild(renderPdaPath(s as PdaState, i)));
    break;
    case "tm": 
      sim.accepted.forEach((s, i) => acceptedEl.appendChild(renderTmPath(s as TmState, i)));
      sim.paths.forEach((s, i) => runningEl.appendChild(renderTmPath(s as TmState, i)));
      sim.rejected.forEach((s, i) => rejectedEl.appendChild(renderTmPath(s as TmState, i)));
    break;
  }
}