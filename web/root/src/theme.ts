import { bus } from "./bus.ts";

const themeBtn = document.getElementById("themeToggle") as HTMLButtonElement;

type Theme = "dark" | "light";

function getPreferredTheme(): Theme {
  // 1) saved preference
  const saved = localStorage.getItem("theme");
  if (saved === "dark" || saved === "light") return saved;

  // 2) OS preference
  const prefersLight = globalThis.window.matchMedia?.(
    "(prefers-color-scheme: light)",
  )?.matches;
  return prefersLight ? "light" : "dark";
}

function setTheme(theme: Theme) {
  document.documentElement.dataset.theme = theme;
  localStorage.setItem("theme", theme);

  // update button label
  themeBtn.textContent = theme === "dark" ? "🌙 Dark" : "☀️ Light";
  
  bus.emit("theme/update", undefined);
}

bus.on("begin", _ => setTheme(getPreferredTheme()))

themeBtn.addEventListener("click", toggleTheme);
function toggleTheme() {
  const current = (document.documentElement.dataset.theme as Theme) || "dark";
  setTheme(current === "dark" ? "light" : "dark");
}

globalThis.window.matchMedia?.("(prefers-color-scheme: light)")
  ?.addEventListener("change", () => {
    if (localStorage.getItem("theme")) return;
    setTheme(getPreferredTheme());
  });



