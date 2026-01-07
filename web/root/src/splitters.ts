type Axis = "x" | "y";

function clamp(n: number, min: number, max: number) {
  return Math.max(min, Math.min(max, n));
}

function parsePx(v: string | null, fallback: number): number {
  if (!v) return fallback;
  const s = v.trim().toLowerCase();
  if (s.endsWith("px")) {
    const n = Number(s.slice(0, -2));
    return Number.isFinite(n) ? n : fallback;
  }
  const n = Number(s);
  return Number.isFinite(n) ? n : fallback;
}

function parsePercent(v: string | null, fallbackPct: number): number {
  if (!v) return fallbackPct;
  const s = v.trim().toLowerCase();
  if (s.endsWith("%")) {
    const n = Number(s.slice(0, -1));
    return Number.isFinite(n) ? n : fallbackPct;
  }
  const n = Number(s);
  return Number.isFinite(n) ? n : fallbackPct;
}

function getCssVar(el: HTMLElement, name: string): string | null {
  const v = getComputedStyle(el).getPropertyValue(name);
  return v ? v.trim() : null;
}

/**
 * Generic rule:
 * - hSplit controls the size of the FIRST pane (top) as a percent of parent height
 * - vSplit controls the size of the THIRD pane (right) as a percent of parent width
 *
 * This matches common editor layouts:
 *   rows: [A][split][B] => A sized, B flex
 *   cols: [A][split][B] => B sized, A flex
 */
export function enableGenericSplitters() {
  enableAll("y", ".hSplit");
  enableAll("x", ".vSplit");
}

function enableAll(axis: Axis, selector: string) {
  for (const splitter of document.querySelectorAll<HTMLElement>(selector)) {
    const parent = splitter.parentElement as HTMLElement | null;
    if (!parent) continue;

    // Require exactly A | splitter | B
    const kids = Array.from(parent.children);
    if (kids.length !== 3 || kids[1] !== splitter) {
      console.warn("Splitter parent must have exactly 3 children: A | splitter | B", parent);
      continue;
    }

    const gap = axis === "y" ? splitter.getBoundingClientRect().height || 8
                             : splitter.getBoundingClientRect().width || 8;

    // Read per-splitter overrides from CSS variables (optional)
    // Defaults:
    //  - default size = 60% (hSplit) or 30% (vSplit)
    //  - minA/minB = 80/180 for hSplit, 220/220 for vSplit
    const defaultPct = parsePercent(
      getCssVar(splitter, "--split-default"),
      axis === "y" ? 60 : 30,
    );

    const minA = parsePx(
      getCssVar(splitter, "--split-min-a"),
      axis === "y" ? 80 : 220,
    );

    const minB = parsePx(
      getCssVar(splitter, "--split-min-b"),
      axis === "y" ? 180 : 220,
    );

    // Make parent a grid automatically (no container classes needed)
    parent.style.display = "grid";
    parent.style.overflow = "hidden";

    // Apply initial template if none set yet
    if (axis === "y") {
      // top sized in %, bottom flex
      if (!parent.style.gridTemplateRows) {
        parent.style.gridTemplateRows = `${defaultPct}% ${gap}px 1fr`;
      }
    } else {
      // right sized in %, left flex
      if (!parent.style.gridTemplateColumns) {
        parent.style.gridTemplateColumns = `1fr ${gap}px ${defaultPct}%`;
      }
    }

    let dragging = false;

    splitter.addEventListener("pointerdown", (e) => {
      dragging = true;
      splitter.setPointerCapture(e.pointerId);
      document.body.style.cursor = axis === "y" ? "row-resize" : "col-resize";
      e.preventDefault();
    });

    splitter.addEventListener("pointermove", (e) => {
      if (!dragging) return;
      const rect = parent.getBoundingClientRect();

      if (axis === "y") {
        // control FIRST pane size (top) by mouse Y
        const y = e.clientY - rect.top;
        const maxA = rect.height - gap - minB;
        const newA = clamp(y, minA, maxA);
        const pct = (newA / rect.height) * 100;
        parent.style.gridTemplateRows = `${pct}% ${gap}px 1fr`;
      } else {
        // control THIRD pane size (right) by distance from right edge
        const xFromRight = rect.right - e.clientX;
        const maxB = rect.width - gap - minA;
        const newB = clamp(xFromRight, minB, maxB);
        const pct = (newB / rect.width) * 100;
        parent.style.gridTemplateColumns = `1fr ${gap}px ${pct}%`;
      }
    });

    splitter.addEventListener("pointerup", (e) => {
      dragging = false;
      document.body.style.cursor = "";
      splitter.releasePointerCapture(e.pointerId);
    });

    splitter.addEventListener("pointercancel", () => {
      dragging = false;
      document.body.style.cursor = "";
    });

    // Optional: keep within bounds on resize (no stored state needed)
    globalThis.window.addEventListener("resize", () => {
      const rect = parent.getBoundingClientRect();
      if (axis === "y") {
        // read current pct from template if possible; otherwise skip
        const parts = (parent.style.gridTemplateRows || "").split(" ");
        if (parts.length >= 3 && parts[0].endsWith("%")) {
          const pct = parseFloat(parts[0]);
          const px = (pct / 100) * rect.height;
          const maxA = rect.height - gap - minB;
          const clampedPx = clamp(px, minA, maxA);
          const clampedPct = (clampedPx / rect.height) * 100;
          parent.style.gridTemplateRows = `${clampedPct}% ${gap}px 1fr`;
        }
      } else {
        const parts = (parent.style.gridTemplateColumns || "").split(" ");
        if (parts.length >= 3 && parts[2].endsWith("%")) {
          const pct = parseFloat(parts[2]);
          const px = (pct / 100) * rect.width;
          const maxB = rect.width - gap - minA;
          const clampedPx = clamp(px, minB, maxB);
          const clampedPct = (clampedPx / rect.width) * 100;
          parent.style.gridTemplateColumns = `1fr ${gap}px ${clampedPct}%`;
        }
      }
    });
  }
}
enableGenericSplitters();