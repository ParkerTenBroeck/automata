function clamp(n: number, min: number, max: number) {
  return Math.max(min, Math.min(max, n));
}

function getVarPx(el: HTMLElement, name: string, fallback: number) {
  const v = getComputedStyle(el).getPropertyValue(name).trim();
  if (!v) return fallback;
  const s = v.toLowerCase();
  if (s.endsWith("px")) {
    const n = Number(s.slice(0, -2));
    return Number.isFinite(n) ? n : fallback;
  }
  const n = Number(s);
  return Number.isFinite(n) ? n : fallback;
}

function getVarPct(el: HTMLElement, name: string, fallback: number) {
  const v = getComputedStyle(el).getPropertyValue(name).trim();
  if (!v) return fallback;
  const s = v.toLowerCase();
  if (s.endsWith("%")) {
    const n = Number(s.slice(0, -1));
    return Number.isFinite(n) ? n : fallback;
  }
  const n = Number(s);
  return Number.isFinite(n) ? n : fallback;
}

function ensureFlexParent(parent: HTMLElement, axis: "row" | "column") {
  // Don't stomp on an existing layout if it's already flex in the right direction
  const cs = getComputedStyle(parent);
  if (cs.display !== "flex") parent.style.display = "flex";
  parent.style.flexDirection = axis;
  parent.style.overflow = "hidden";
}

function ensurePaneCanShrink(pane: HTMLElement) {
  // Critical for nested flex layouts (otherwise children overflow)
  pane.style.minWidth = "0";
  pane.style.minHeight = "0";
}

function setFixedSize(
  pane: HTMLElement,
  axis: "x" | "y",
  px: number,
) {
  // For flex: fixed pane should not grow/shrink, basis = px
  pane.style.flexGrow = "0";
  pane.style.flexShrink = "0";
  pane.style.flexBasis = `${px}px`;

  // Helps some browsers respect size
  if (axis === "x") {
    pane.style.width = `${px}px`;
  } else {
    pane.style.height = `${px}px`;
  }
}

function setFlexFill(pane: HTMLElement) {
  // Fill remaining space
  pane.style.flex = "1 1 auto";
}

function enableFlexSplitters() {
  // Horizontal: A | hSplit | B (top/split/bottom)
  for (const splitter of document.querySelectorAll<HTMLElement>(".hSplit:not(.styleOnly)")) {
    const parent = splitter.parentElement as HTMLElement | null;
    if (!parent) continue;

    const kids = Array.from(parent.children) as HTMLElement[];
    if (kids.length !== 3 || kids[1] !== splitter) {
      console.warn("hSplit parent must be A | splitter | B", parent);
      continue;
    }

    const a = kids[0];
    const b = kids[2];

    ensureFlexParent(parent, "column");
    ensurePaneCanShrink(a);
    ensurePaneCanShrink(b);
    setFlexFill(b); // bottom fills

    const gap = splitter.getBoundingClientRect().height || 8;
    splitter.style.flex = `0 0 ${gap}px`;

    // Optional per-splitter CSS vars:
    // --split-default: 60%  (of parent height)
    // --split-min-a: 80px
    // --split-min-b: 180px
    const defPct = getVarPct(splitter, "--split-default", 60);
    const minA = getVarPx(splitter, "--split-min-a", 220);
    const minB = getVarPx(splitter, "--split-min-b", 220);

    // Set initial size (A is fixed)
    {
      const r = parent.getBoundingClientRect();
      const px = clamp((defPct / 100) * r.height, minA, r.height - gap - minB);
      setFixedSize(a, "y", px);
    }

    let dragging = false;

    splitter.addEventListener("pointerdown", (e) => {
      dragging = true;
      splitter.setPointerCapture(e.pointerId);
      document.body.style.cursor = "row-resize";
      e.preventDefault();
    });

    splitter.addEventListener("pointermove", (e) => {
      if (!dragging) return;
      const r = parent.getBoundingClientRect();
      const y = e.clientY - r.top;

      const maxA = r.height - gap - minB;
      const newA = clamp(y, minA, maxA);
      setFixedSize(a, "y", newA);
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
  }

  // Vertical: A | vSplit | B (left/split/right)
  for (const splitter of document.querySelectorAll<HTMLElement>(".vSplit:not(.styleOnly)")) {
    const parent = splitter.parentElement as HTMLElement | null;
    if (!parent) continue;

    const kids = Array.from(parent.children) as HTMLElement[];
    if (kids.length !== 3 || kids[1] !== splitter) {
      console.warn("vSplit parent must be A | splitter | B", parent);
      continue;
    }

    const a = kids[0];
    const b = kids[2];

    ensureFlexParent(parent, "row");
    ensurePaneCanShrink(a);
    ensurePaneCanShrink(b);
    setFlexFill(a); // left fills

    const gap = splitter.getBoundingClientRect().width || 8;
    splitter.style.flex = `0 0 ${gap}px`;


    const defPct = getVarPct(splitter, "--split-default", 50);
    const minA = getVarPx(splitter, "--split-min-a", 220);
    const minB = getVarPx(splitter, "--split-min-b", 220);

    // Set initial size (B is fixed)
    {
      const r = parent.getBoundingClientRect();
      const px = clamp((defPct / 100) * r.width, minB, r.width - gap - minA);
      setFixedSize(b, "x", px);
    }

    let dragging = false;

    splitter.addEventListener("pointerdown", (e) => {
      dragging = true;
      splitter.setPointerCapture(e.pointerId);
      document.body.style.cursor = "col-resize";
      e.preventDefault();
    });

    splitter.addEventListener("pointermove", (e) => {
      if (!dragging) return;
      const r = parent.getBoundingClientRect();

      // Right pane width = distance from right edge
      const xFromRight = r.right - e.clientX;

      const maxB = r.width - gap - minA;
      const newB = clamp(xFromRight, minB, maxB);
      setFixedSize(b, "x", newB);
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
  }
}
enableFlexSplitters();
enableFlexSplitters();