import {nodes, edges, network} from "./visualizer.ts"

const togglePhysicsBtn = document.getElementById("togglePhysics") as HTMLButtonElement;
const resetLayoutBtn   = document.getElementById("resetLayout")   as HTMLButtonElement;
const playPauseBtn     = document.getElementById("playPause")     as HTMLButtonElement;
const stepBtn          = document.getElementById("step")          as HTMLButtonElement;
const speedSlider      = document.getElementById("speed")         as HTMLInputElement;
const speedLabel       = document.getElementById("speedLabel")    as HTMLSpanElement;
const resetSimBtn      = document.getElementById("resetSim") as HTMLButtonElement;


function stepSimulation(): void {
  console.log("step");
}

function resetSimulation(): void {
  console.log("reset");
}

// ---- Physics toggle (styled label) ----
function setPhysicsButtonUI(enabled: boolean) {
  togglePhysicsBtn.classList.toggle("active", enabled);
  togglePhysicsBtn.textContent = enabled ? "Physics: ON" : "Physics: OFF";
}

togglePhysicsBtn.onclick = () => {
  const enabled = !togglePhysicsBtn.classList.contains("active");
  setPhysicsButtonUI(enabled);
  network.setOptions({ physics: { enabled } });
};

setPhysicsButtonUI(togglePhysicsBtn.classList.contains("active"));

resetLayoutBtn.onclick = () => {
  try {
      nodes.forEach((n) => {
        n.physics = true;
        n.x = undefined;
        n.y = undefined;
      });
      network.setData({ nodes, edges });
  } catch {
    // Last resort
    network.setData({ nodes, edges });
  }

  // If physics button is OFF, keep it OFF (don’t surprise the user)
  const physicsEnabled = togglePhysicsBtn.classList.contains("active");
  network.setOptions({ physics: { enabled: physicsEnabled } });
};

// ---- Play/Pause + Speed ----
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
    // If your step can throw, keep the interval alive:
    try { stepSimulation(); } catch (e) { console.error(e); }
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

stepBtn.onclick = () => {
  stepSimulation();
};

resetSimBtn.onclick = () => {
  // Stop if running
  if (running) setRunning(false);

  // Reset
  resetSimulation();

  // Optional: re-enable Step after reset
  stepBtn.disabled = false;
};