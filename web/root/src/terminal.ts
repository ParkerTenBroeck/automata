// deno-lint-ignore-file

import {
  ViewPlugin,
} from "npm:@codemirror/view";

import { analysisField } from "./editor.ts";

function escapeHtml(s: string) {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}


function ansiToHtml(input: string) {
  // deno-lint-ignore no-control-regex
  const ESC_RE = /\x1b\[([0-9;]*)m/g;

  let out = "";
  let lastIndex = 0;

  // current style state
  let fg: number|null = null; // e.g. 31, 92
  let bg: number|null = null; // e.g. 41
  let bold = false;
  let dim = false;

  function openSpanIfNeeded(text: string) {
    if (text.length === 0) return "";
    const classes = [];
    if (bold) classes.push("ansi-bold");
    if (dim) classes.push("ansi-dim");
    if (fg != null) classes.push(`ansi-fg-${fg}`);
    if (bg != null) classes.push(`ansi-bg-${bg}`);
    if (classes.length === 0) return escapeHtml(text);
    return `<span class="${classes.join(" ")}">${escapeHtml(text)}</span>`;
  }

  function applyCodes(codes: string[]) {
    if (codes.length === 0) codes = ["0"];
    for (const c of codes) {
      const code = Number(c);
      if (Number.isNaN(code)) continue;

      if (code === 0) {
        fg = null; bg = null; bold = false; dim = false;
      } else if (code === 1) {
        bold = true;
      } else if (code === 2) {
        dim = true;
      } else if (code === 22) {
        bold = false; dim = false;
      } else if (code === 39) {
        fg = null;
      } else if (code === 49) {
        bg = null;
      } else if ((code >= 30 && code <= 37) || (code >= 90 && code <= 97)) {
        fg = code;
      } else if ((code >= 40 && code <= 47) || (code >= 100 && code <= 107)) {
        bg = code;
      }
    }
  }

  let m;
  while ((m = ESC_RE.exec(input)) !== null) {
    const chunk = input.slice(lastIndex, m.index);
    out += openSpanIfNeeded(chunk);

    const codes = m[1] ? m[1].split(";") : [];
    applyCodes(codes);

    lastIndex = ESC_RE.lastIndex;
  }

  out += openSpanIfNeeded(input.slice(lastIndex));
  return out;
}

  // @ts-expect-error bad library
function formatTerminal(view) {
  const term = document.getElementById("terminal");
  if (!term) return;

  const { log, log_formatted } = view.state.field(analysisField);

  let s = "";
  s += `\x1b[90m[compile]\x1b[0m ${log.length} diagnostics\n`;

  term.innerHTML = ansiToHtml(s + log_formatted);
}

export const terminalPlugin = ViewPlugin.fromClass(
  class {

    // @ts-expect-error bad library
    constructor(view) {
    // @ts-expect-error bad library
      this.view = view;
      formatTerminal(view);
    }
    // @ts-expect-error bad library
    update(update) {
      if (update.docChanged) formatTerminal(update.view);
    }
  }
);