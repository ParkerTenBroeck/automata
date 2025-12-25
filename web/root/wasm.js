console.debug("Loading wasm…");
import init, *  as wasm from "./automata/automata_web.js";
try{
    console.debug("Wasm loaded. Starting app…");
    window.wasm = wasm;
    await init();
    console.debug("App started.");
    document.getElementById("center_text").innerHTML = '';
    document.getElementById("app").style.display = '';
    wasm.init();
}catch(e){
    console.error("Failed to start: " + error);
    document.getElementById("the_canvas_id").remove();
    document.getElementById("center_text").innerHTML = `
        <p>
            An error occurred during loading:
        </p>
        <p style="font-family:Courier New">
            ${error}
        </p>
        <p style="font-size:14px">
            Make sure you use a modern browser with WebGL and WASM enabled.
        </p>`;
}

export default wasm