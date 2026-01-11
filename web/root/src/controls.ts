import { bus } from "./bus.ts";

const togglePhysicsBtn = document.getElementById(
  "togglePhysics",
) as HTMLButtonElement;
const resetLayoutBtn = document.getElementById(
  "resetLayout",
) as HTMLButtonElement;
const playPauseBtn = document.getElementById(
  "playPauseSim",
) as HTMLButtonElement;
const stepBtn = document.getElementById("stepSim") as HTMLButtonElement;
const speedSlider = document.getElementById("speedSim") as HTMLInputElement;
const speedLabel = document.getElementById("speedSimLabel") as HTMLSpanElement;
const reloadSimBtn = document.getElementById("reloadSim") as HTMLButtonElement;
const clearSimBtn = document.getElementById("clearSim") as HTMLButtonElement;

bus.on("controls/physics", ({ enabled }) => {
  togglePhysicsBtn.classList.toggle("active", enabled);
  togglePhysicsBtn.textContent = enabled ? "Physics: ON" : "Physics: OFF";
});

togglePhysicsBtn.onclick = () => {
  const enabled = !togglePhysicsBtn.classList.contains("active");
  bus.emit("controls/physics", { enabled });
};

bus.emit("controls/physics", {
  enabled: togglePhysicsBtn.classList.contains("active"),
});

resetLayoutBtn.onclick = () => bus.emit("controls/reset_network", undefined);

clearSimBtn.onclick = () => bus.emit("controls/clear_simulation", undefined);

stepBtn.onclick = () => {
  bus.emit("controls/step_simulation", undefined);
};

reloadSimBtn.onclick = () => bus.emit("controls/reload_simulation", undefined);

function updateButtons() {
  stepBtn.disabled = !simulation_active || running;
  playPauseBtn.disabled = !simulation_active;
  clearSimBtn.disabled = !simulation_active;
}

bus.on("controls/reload_simulation", (_) => {
  if (running) setRunning(false);
  updateButtons();
});

bus.on("automata/sim/update", ({ simulation }) => {
  simulation_active = !!simulation;
  if (!simulation) {
    if (running) setRunning(false);
  }
  updateButtons();
});

bus.on("automata/sim/after_step", ({ result }) => {
  if (result !== "pending") {
    if (running) setRunning(false);
  }
});

let simulation_active = false;
let running = false;
let timer: number | null = null;

// speed slider is "steps per second"
function getStepsPerSecond() {
  return Math.max(1, Math.min(60, Number(speedSlider.value) || 10));
}
function updateSpeedUI() {
  speedLabel.textContent = `${getStepsPerSecond()}×`;
}
updateSpeedUI();

speedSlider.addEventListener("input", () => {
  updateSpeedUI();
  if (running) restartTimer();
});

function stopTimer() {
  if (timer !== null) {
    clearInterval(timer);
    timer = null;
  }
}

function restartTimer() {
  stopTimer();
  const sps = getStepsPerSecond();
  const intervalMs = Math.round(1000 / sps);

  timer = globalThis.window.setInterval(() => {
    bus.emit("controls/step_simulation", undefined);
  }, intervalMs);
}

function setRunning(on: boolean) {
  running = on;
  playPauseBtn.textContent = running ? "⏸ Pause" : "▶ Play";
  playPauseBtn.classList.toggle("btn-primary", !running);
  playPauseBtn.classList.toggle("btn-secondary", running);

  // Disable step while running (optional, but feels nice)
  stepBtn.disabled = running;

  if (running) restartTimer();
  else stopTimer();
}

playPauseBtn.onclick = () => setRunning(!running);
