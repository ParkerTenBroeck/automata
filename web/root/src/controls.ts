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

// speed slider is "steps per second"
function getStepsPerSecond() {
  return Math.max(1, Math.min(60, Number(speedSlider.value) || 10));
}
function updateSpeedUI() {
  speedLabel.textContent = `${getStepsPerSecond()}×`;
}
updateSpeedUI();

class Controls {
  static simulation_active = false;
  static running = false;
  static timer: number | null = null;

  static updateButtons() {
    stepBtn.disabled = !Controls.simulation_active || Controls.running;
    playPauseBtn.disabled = !Controls.simulation_active;
    clearSimBtn.disabled = !Controls.simulation_active;
  }
  static setRunning(on: boolean) {
    Controls.running = on;
    playPauseBtn.textContent = Controls.running ? "⏸ Pause" : "▶ Play";
    playPauseBtn.classList.toggle("btn-primary", !Controls.running);
    playPauseBtn.classList.toggle("btn-secondary", Controls.running);

    if (Controls.running) Controls.restartTimer();
    else Controls.stopTimer();
    Controls.updateButtons();
  }
  static stop() {
    if (Controls.running) Controls.setRunning(false);
  }
  static stopTimer() {
    if (Controls.timer !== null) {
      clearInterval(Controls.timer);
      Controls.timer = null;
    }
  }

  static restartTimer() {
    Controls.stopTimer();
    const sps = getStepsPerSecond();
    const intervalMs = Math.round(1000 / sps);

    Controls.timer = globalThis.window.setInterval(() => {
      bus.emit("controls/sim/step", undefined);
    }, intervalMs);
  }

  static {
    speedSlider.addEventListener("input", () => {
      updateSpeedUI();
      if (Controls.running) Controls.restartTimer();
    });
    playPauseBtn.onclick = () => Controls.setRunning(!Controls.running);
    resetLayoutBtn.onclick = () =>
      bus.emit("controls/vis/reset_network", undefined);
    clearSimBtn.onclick = () => bus.emit("controls/sim/clear", undefined);
    stepBtn.onclick = () => bus.emit("controls/sim/step", undefined);
    reloadSimBtn.onclick = () => bus.emit("controls/sim/reload", undefined);
    togglePhysicsBtn.onclick = () => {
      const enabled = !togglePhysicsBtn.classList.contains("active");
      bus.emit("controls/vis/physics", { enabled });
    };

    bus.on("controls/vis/physics", ({ enabled }) => {
      togglePhysicsBtn.classList.toggle("active", enabled);
      togglePhysicsBtn.textContent = enabled ? "Physics: ON" : "Physics: OFF";
    });

    bus.on("controls/sim/reload", (_) => {
      if (Controls.running) Controls.setRunning(false);
    });

    bus.on("automata/sim/update", simulation => {
      Controls.simulation_active = !!simulation;
      if (!simulation) Controls.stop();
    });

    bus.on("automata/sim/after_step", ({ result }) => {
      if (result !== "pending") Controls.stop();
    });

    bus.emit("controls/vis/physics", {
      enabled: togglePhysicsBtn.classList.contains("active"),
    });
  }
}
