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

console.log("clean dist...");
await Deno.remove(DIST, { recursive: true }).catch(() => {});
await Deno.mkdir(DIST, { recursive: true });


console.log("copy assets");
await copyFolder(new URL("assets/", ROOT), DIST);

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



export async function copyFolder(
  srcDir: URL,
  destDir: URL,
): Promise<void> {
  await Deno.mkdir(destDir, { recursive: true });

  for await (const entry of Deno.readDir(srcDir)) {
    const srcPath = new URL(entry.name, srcDir.href);
    const destPath = new URL(entry.name, destDir.href);
  
    if (entry.isDirectory) {
      await copyFolder(srcPath, destPath);
    } else if (entry.isFile) {
      await Deno.copyFile(srcPath, destPath);
    }
  }
}