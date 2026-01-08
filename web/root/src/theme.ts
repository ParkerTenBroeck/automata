import { invalidateGraphThemeCache, network } from "./visualizer.ts";

function cssVar(name: string, fallback = ""): string {
  return getComputedStyle(document.documentElement)
    .getPropertyValue(name)
    .trim() || fallback;
}

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
  applyGraphTheme();
}

function toggleTheme() {
  const current = (document.documentElement.dataset.theme as Theme) || "dark";
  setTheme(current === "dark" ? "light" : "dark");
}

// init
setTheme(getPreferredTheme());

// click handler
themeBtn.addEventListener("click", toggleTheme);

// optional: respond to OS theme changes (only if user hasn't chosen a theme)
globalThis.window.matchMedia?.("(prefers-color-scheme: light)")
  ?.addEventListener("change", () => {
    if (localStorage.getItem("theme")) return; // user has chosen, don't override
    setTheme(getPreferredTheme());
  });

function applyGraphTheme() {
  invalidateGraphThemeCache();

  network.setOptions({
    nodes: {
      font: {
        color: cssVar("--graph-node-text"),
      },
    },
    edges: {
      labelHighlightBold: true,
      font: { 
        align: "middle", 
        color: cssVar("--fg-0"),
        strokeColor: cssVar("--bg-0"),
        bold: {
          color: cssVar("--fg-1"),
          mod: ''
        },
      },
      color: {
        color: cssVar("--graph-edge"),
        highlight: cssVar("--graph-edge-active"),
        hover: cssVar("--graph-edge-hover"),
      },
      shadow: {
        enabled: true,
        color: cssVar("--bg-2")
      }
    },
  });
}




