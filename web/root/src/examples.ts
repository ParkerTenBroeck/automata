import { setText } from "./editor.ts";

export type Category =
  | "Tutorial"
  | "DFA"
  | "NFA"
  | "DPDA"
  | "NPDA"
  | "TM"
  | "NTM";

export class Example {
  category: Category;
  title: string;
  machine: string;

  constructor(category: Category, title: string, machine: string) {
    this.category = category;
    this.title = title;
    this.machine = machine;
  }
}

export const examples: Example[] = [
  new Example(
    "Tutorial",
    "DFA",
    `// strings over a,b which start and end with different letters

type = DFA                     // type of machine DFA, NFA, DPDA, NPDA, DTM, NTM 
Q    = {q0, qa, qa', qb, qb'}  // set of states 
E    = {a, b}                  // alphabet
F    = {qa', qb'}              // set of final states
q0   = q0                      // initial state

// transition function (state, letter) -> state
d(q0, a) = qa
d(q0, b) = qb

d(qa, a) = qa
d(qa, b) = qa'

d(qa', a) = qa
d(qa', b) = qa'

d(qb, a) = qb'
d(qb, b) = qb

d(qb', a) = qb'
d(qb', b) = qb`,
  ),

  new Example(
    "DFA",
    "modulo",
    `type=DFA
E={1,2,3}
Q={q0, q1, q2, q3, q4}
F = {q0}
q0=q0

d(q0, 1) = q1
d(q1, 1) = q2
d(q2, 1) = q3
d(q3, 1) = q4
d(q4, 1) = q0

d(q0, 2) = q2
d(q1, 2) = q3
d(q2, 2) = q4
d(q3, 2) = q0
d(q4, 2) = q1

d(q0, 3) = q3
d(q1, 3) = q4
d(q2, 3) = q0
d(q3, 3) = q1
d(q4, 3) = q2`,
  ),

  new Example(
    "DPDA",
    "unequal",
    `type=DPDA
Q = {q0, qas, qeq, qmb, qlb} // states
E = {a, b}                   // alphabet
T = {z0, A}                  // stack
F = {qmb, qlb}               // final states
q0 = q0
z0 = z0

d(q0, a, z0) = (qas, z0)

d(qas, a, z0) = (qas, [A z0])
d(qas, b, z0) = (qeq, z0)
d(qas, a, A) = (qas, [A A])
d(qas, b, A) = (qlb, ~)

d(qlb, b, A) = (qeq, ~)
d(qlb, b, z0) = (qeq, z0)

d(qeq, b, z0) = (qmb, z0)

d(qmb, b, z0) = (qmb, z0)`,
  ),

  new Example(
    "NPDA",
    "unequal",
    `type=NPDA
Q = {q0, q1} // states
E = {a, b} // alphabet
T = {z0, A, B} // stack
q0 = q0
z0 = z0

// construct all possible permutations of A's and B's
d(q0, epsilon, z0)  =   { (q0, [A z0]), (q0, [B z0]) }
d(q0, epsilon, A)   =   { (q0, [A A]),  (q0, [B A])  }

d(q0, epsilon, B)   =   { (q0, [A B]),  (q0, [B B])  }

// transition to q1
d(q0, epsilon, z0)  =   { (q1, z0) }
d(q0, epsilon, A)   =   { (q1, A)  }
d(q0, epsilon, B)   =   { (q1, B)  }

// consume stack until empty
d(q1, a, A)         =   { (q1, epsilon) }
d(q1, b, B)         =   { (q1, epsilon) }`,
  ),
];

const CATEGORY_ORDER: Category[] = [
  "Tutorial",
  "DFA",
  "NFA",
  "DPDA",
  "NPDA",
  "TM",
  "NTM",
];

function buildExamplesDropdown(
  selectEl: HTMLSelectElement,
  examples: Example[],
  onPick?: (ex: Example) => void,
) {
  // Clear everything except the first placeholder option (if present)
  const keepFirstPlaceholder = selectEl.options.length > 0 &&
    selectEl.options[0].disabled && selectEl.options[0].value === "";

  selectEl.innerHTML = "";
  if (keepFirstPlaceholder) {
    const placeholder = document.createElement("option");
    placeholder.value = "";
    placeholder.disabled = true;
    placeholder.selected = true;
    placeholder.textContent = "Choose an example…";
    selectEl.appendChild(placeholder);
  }

  // Group examples by category
  const grouped = new Map<Category, Example[]>();
  for (const ex of examples) {
    if (!grouped.has(ex.category)) grouped.set(ex.category, []);
    grouped.get(ex.category)!.push(ex);
  }

  // Optional: sort titles within each group
  for (const [cat, list] of grouped) {
    list.sort((a, b) => a.title.localeCompare(b.title));
    grouped.set(cat, list);
  }

  // Create optgroups in your preferred order (and then any extras)
  const categoriesToRender: Category[] = [
    ...CATEGORY_ORDER.filter((c) => grouped.has(c)),
    ...Array.from(grouped.keys()).filter((c) => !CATEGORY_ORDER.includes(c))
      .sort(),
  ];

  // We'll store a stable reference via an index into the examples array
  // (simplest + avoids encoding large machine strings into <option value>)
  const indexByIdentity = new Map<Example, number>();
  examples.forEach((ex, i) => indexByIdentity.set(ex, i));

  for (const category of categoriesToRender) {
    const optgroup = document.createElement("optgroup");
    optgroup.label = category;

    for (const ex of grouped.get(category)!) {
      const opt = document.createElement("option");
      opt.value = String(indexByIdentity.get(ex)!); // index
      opt.textContent = ex.title;
      optgroup.appendChild(opt);
    }

    selectEl.appendChild(optgroup);
  }

  // Change handler
  selectEl.onchange = () => {
    const v = selectEl.value;
    if (!v) return;
    const picked = examples[Number(v)];
    if (picked && onPick) onPick(picked);
    selectEl.value = "";
  };
}

const selectEl = document.getElementById("exampleSelect") as HTMLSelectElement;
buildExamplesDropdown(selectEl, examples, (picked) => {
  setText(picked.machine);
});
