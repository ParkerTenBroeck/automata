import { bus } from "./bus.ts";

export type Category =
  | "Tutorial"
  | "DFA"
  | "NFA"
  | "DPDA"
  | "NPDA"
  | "TM"
  | "NTM"
  | "CFG";

export class Example {
  readonly category: Category;
  readonly title: string;
  readonly machine: string;

  constructor(category: Category, title: string, machine: string) {
    this.category = category;
    this.title = title;
    this.machine = machine;
  }
}

export const examples: readonly Example[] = [
  new Example(
    "Tutorial",
    "DFA",
    `// strings over a,b which start and end with different letters

type   = DFA                     // type of machine DFA, NFA, DPDA, NPDA, DTM, NTM 
Q      = {q0, qa, qa', qb, qb'}  // set of states 
E      = {a, b}                  // alphabet
F      = {qa', qb'}              // set of final states
q0     = q0                      // initial state

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
    "Tutorial",
    "NFA",
    `// strings of 1's whos length is divisible by two or three and longer than 1

type   = NFA                          // type of machine
Q      = {q0, q2, q2f, q3, q3', q3f}  // set of states 
E      = {1}                          // alphabet
F      = {q2f, q3f}                   // set of final states
q0     = q0                           // initial state

// transition function (state, letter) -> state

// non deterministic part
d(q0, 1) = q2
d(q0, 1) = q3

d(q2, 1) = q2f
d(q2f, 1) = q2

d(q3, 1) = q3'
d(q3', 1) = q3f
d(q3f, 1) = q3
`,
  ),

  new Example(
    "Tutorial",
    "NFA with epsilon",
    `// strings containing only all a's, or all b's, or all c's

type   = NFA               // type of machine
Q      = {q0, qa, qb, qc}  // set of states 
E      = {a, b, c}         // alphabet
F      = {qa, qb, qc}      // set of final states
q0     = q0                // initial state

// transition function (state, letter) -> state

// non deterministic part
d(q0, epsilon) = qa
d(q0, epsilon) = qb
d(q0, epsilon) = qc

d(qa, a) = qa
d(qb, b) = qb
d(qc, c) = qc
`,
  ),

  new Example(
    "Tutorial",
    "DPDA Final State",
    `// Accept strings over a,b of the form a^nb^k n != k n,k > 0

type   = DPDA
Q      = {q0, qas, qeq, qmb, qlb} // states
E      = {a, b}                   // alphabet
T      = {z0, A}                  // stack
F      = {qmb, qlb}               // final states
accept = F                        // accept by final state
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
    "Tutorial",
    "DPDA Empty Stack",
    `// Accept strings over a,b which are of form a^n b^n 

type   = DPDA
Q      = {qa, qb} // states
E      = {a, b}       // alphabet
T      = {Z0, A}      // stack
accept = N            // accept by empty stack
q0 = qa
z0 = Z0


// build stack of A's (equal to a's encountered)
d(qa, a, Z0) = (qa, [A Z0])
d(qa, a, A) = (qa, [A A])

// transition to b state once a b is encountered
d(qa, b, A) = (qb, ~)

// consume b's until A's run out
d(qb, b, A) = (qb, ~)`,
  ),

  // new Example(
  //   "Tutorial",
  //   "NPDA Final State",
  //   ``,
  // ),

  new Example(
    "Tutorial",
    "NPDA Empty Stack",
    `// Accept all strings over a,b which are spelt the same backwards and forwards

type=NPDA
Q = {q0, q1}   // states
E = {a, b}     // alphabet
T = {Z0, A, B} // stack
accept = E     // accept by empty stack
q0 = q0
z0 = Z0

// push letters we see to stack
d(q0, a, Z0)  =   (q0, [A Z0])
d(q0, b, Z0)  =   (q0, [B Z0])

d(q0, a, A)  =   (q0, [A A])
d(q0, b, A)  =   (q0, [B A])

d(q0, a, B)  =   (q0, [A B])
d(q0, b, B)  =   (q0, [B B])

// transition to q1 
// even
d(q0, epsilon, Z0)  =   { (q1, Z0) }
d(q0, epsilon, A)   =   { (q1, A)  }
d(q0, epsilon, B)   =   { (q1, B)  }
// odd
d(q0, a, Z0)  =   { (q1, Z0) }
d(q0, a, A)   =   { (q1, A)  }
d(q0, a, B)   =   { (q1, B)  }

d(q0, b, Z0)  =   { (q1, Z0) }
d(q0, b, A)   =   { (q1, A)  }
d(q0, b, B)   =   { (q1, B)  }

// consume stack until empty
d(q1, a, A)         =   { (q1, epsilon) }
d(q1, b, B)         =   { (q1, epsilon) }`,
  ),

  new Example(
    "Tutorial",
    "TM",
    `// Accept strings over a,b,c of the form a^n b^n c^n, n > 0

type   = TM
Q      = {q1, q2, q3, q4, q5, q6, qf} // states
T      = {a, b,c, B, X, Y, Z}         // stack
F      = {qf}                         // final states
q0 = q1
B = B

d(q1, a) = (q2, X, R)
d(q1, Y) = (q5, Y, R)

d(q2, a) = (q2, a, R)
d(q2, b) = (q3, Y, R)
d(q2, Y) = (q2, Y, R)

d(q3, b) = (q3, b, R)
d(q3, c) = (q4, Z, L)
d(q3, Z) = (q3, Z, R)

d(q4, a) = (q4, a, L)
d(q4, b) = (q4, b, L)
d(q4, X) = (q1, X, R)
d(q4, Y) = (q4, Y, L)
d(q4, Z) = (q4, Z, L)

d(q5, Y) = (q5, Y, R)
d(q5, Z) = (q6, Z, R)

d(q6, B) = (qf, B, R)
d(q6, Z) = (q6, Z, R)`,
  ),

  // new Example(
  //   "Tutorial",
  //   "NTM",
  //   ``,
  // ),

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
    `type   = DPDA
Q      = {q0, qas, qeq, qmb, qlb} // states
E      = {a, b}                   // alphabet
T      = {z0, A}                  // stack
F      = {qmb, qlb}               // final states
accept = F                        // accept by final state
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
    "palindrome",
    `type=NPDA
Q = {q0, q1} // states
E = {a, b} // alphabet
T = {z0, A, B} // stack
accept = E // accept by empty stack
q0 = q0
z0 = z0

// push letters we see to stack
d(q0, a, z0)  =   (q0, [A z0])
d(q0, b, z0)  =   (q0, [B z0])

d(q0, a, A)  =   (q0, [A A])
d(q0, b, A)  =   (q0, [B A])

d(q0, a, B)  =   (q0, [A B])
d(q0, b, B)  =   (q0, [B B])

// transition to q1 
// even
d(q0, epsilon, z0)  =   { (q1, z0) }
d(q0, epsilon, A)   =   { (q1, A)  }
d(q0, epsilon, B)   =   { (q1, B)  }
// odd
d(q0, a, z0)  =   { (q1, z0) }
d(q0, a, A)   =   { (q1, A)  }
d(q0, a, B)   =   { (q1, B)  }

d(q0, b, z0)  =   { (q1, z0) }
d(q0, b, A)   =   { (q1, A)  }
d(q0, b, B)   =   { (q1, B)  }

// consume stack until empty
d(q1, a, A)         =   { (q1, epsilon) }
d(q1, b, B)         =   { (q1, epsilon) }`,
  ),

  new Example(
    "NPDA",
    "kleen star stack",
    `type=NPDA
Q = {q0, q1} // states
E = {a, b} // alphabet
T = {z0, A, B} // stack
accept = E // accept by empty stack
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

  new Example("TM", "a^nb^n",
    `// accepts all strings on {a,b}+ of the form anbn

type = TM
Q = { q0, q1, q2, q3, q4 } // set of internal states
F = { q4 }                 // set of final states
T = { a, b, X, Y, B }      // tape alphabet
B = B                      // the blank symbol (tape initializer symbol)
q0 = q0                    // initial state

d(q0,a)=(q1,x,R)
d(q1,a)=(q1,a,R)
d(q1,Y)=(q1,y,R)
d(q1,b)=(q2,y,L) 

d(q2,Y)=(q2,y,L)
d(q2,a)=(q2,a,L)
d(q2,X)=(q0,x,R)

d(q0,Y)=(q3,y,R)
d(q3,Y)=(q3,y,R)
d(q3,B)=(q4,B,R)
`),

//   new Example("CFG", "definition",
//     `// CFG's aren't supported yet, and this definition is not complete.
// // This is the definition for the grammar the definition has itself

// type=CFG

// S -> TopLevel | TopLevel S

// TopLevel -> Ident "=" Item // Item 
// TopLevel -> Ident Tuple "=" Item // Transition Functions
// TopLevel -> Production | Table
 
// Item -> Symbol | String | Tuple | List

// Symbol -> Ident | "~"
// String -> "\"" "\""
// Tuple -> "(" ItemList ")"
// List -> "{" ItemList "}" | "[" ItemList "]"

// ItemList -> ~ | Item ItemList | Item "," ItemList

// Production -> ProductionGroup "->" ProductionGroupList
// ProductionGroupList -> ProductionGroup | ProductionGroupList "|" ProductionGroup 
// ProductionGroup -> ProductionUnit | ProductionGroup ProductionUnit 
// ProductionUnit -> Ident | "~" | String


// `)
];

const CATEGORY_ORDER: Category[] = [
  "Tutorial",
  "DFA",
  "NFA",
  "DPDA",
  "NPDA",
  "TM",
  "NTM",
  "CFG",
];

function buildExamplesDropdown(
  selectEl: HTMLSelectElement,
  examples: readonly Example[],
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
    // list.sort((a, b) => a.title.localeCompare(b.title));
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
buildExamplesDropdown(selectEl, examples, (example) => {
  bus.emit("example/selected", example);
});
