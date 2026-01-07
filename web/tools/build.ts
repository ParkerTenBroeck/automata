import * as sass from "sass";

// tools/build.ts
const ROOT = new URL("../root/", import.meta.url);
const WASM = new URL("../wasm/", import.meta.url);
const DIST = new URL("../dist/", import.meta.url);

async function run(cmd: string[], cwd?: string) {
  const p = new Deno.Command(cmd[0], { args: cmd.slice(1), cwd });
  const out = await p.output();
  if (!out.success) {
    console.error(new TextDecoder().decode(out.stderr));
    Deno.exit(out.code);
  }
}

// Clean dist
await Deno.remove(DIST, { recursive: true }).catch(() => {});
await Deno.mkdir(DIST, { recursive: true });


console.log("compiling scss...");
const result = sass.compile(String(new URL("style/style.scss", ROOT).pathname), {
  style: "compressed",
});
await Deno.writeTextFile(new URL("style.css", DIST), result.css);

console.log("Compiling wasm lib...");
await run(["wasm-pack", "build", "--target", "web", "--release", "--out-dir", "wasm"], "");
await Deno.copyFile(new URL("automata_web_bg.wasm", WASM), new URL("automata_web_bg.wasm", DIST));

console.log("Compiling bundle...");
const bundle = new Deno.Command(Deno.execPath(), {
  args: ["bundle", "--platform=browser", "--outdir", "dist", "root/index.html", "--minify",],
});
const bundleRes = await bundle.output();
if (!bundleRes.success) {
  console.error(new TextDecoder().decode(bundleRes.stderr));
  Deno.exit(bundleRes.code);
}

console.log("Build complete: dist/");
