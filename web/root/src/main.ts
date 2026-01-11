import { bus } from "./bus.ts";
import "./splitters.ts"
import "./controls.ts"
import "./theme.ts"
import "./share.ts"
import "./examples.ts"
import "./visualizer.ts"
import "./editor.ts"
import "./simulation.ts"

bus.emit("begin", undefined);