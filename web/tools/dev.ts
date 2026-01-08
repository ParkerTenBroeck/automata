// tools/dev.ts
const BUILD_CMD = ["deno", "run", "-A", "tools/build.ts"];
const SERVE_CMD = [
  "deno",
  "run",
  "--allow-net",
  "--allow-read",
  "--allow-sys",
  "jsr:@std/http/file-server",
  "dist",
];

let building = false;
let server: Deno.ChildProcess | null = null;

async function runBuild() {
  if (building) return;
  building = true;

  console.log("🔨 building…");
  const p = new Deno.Command(BUILD_CMD[0], {
    args: BUILD_CMD.slice(1),
  });
  const r = await p.output();
  if (!r.success) {
    console.error(new TextDecoder().decode(r.stderr));
  } else {
    console.log("✅ build complete");
  }

  building = false;
}

async function startServer() {
  if (server) return;
  const p = new Deno.Command(SERVE_CMD[0], {
    args: SERVE_CMD.slice(1),
    stdout: "inherit",
    stderr: "inherit",
  });
  server = p.spawn();
}

await runBuild();
await startServer();

console.log("👀 watching for changes…");

const watcher = Deno.watchFs(["root", "src"]);
for await (const event of watcher) {
  if (
    event.kind === "modify" ||
    event.kind === "create" ||
    event.kind === "remove"
  ) {
    await runBuild();
  }
}
