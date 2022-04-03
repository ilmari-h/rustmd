import * as WASM from "./rustmd.js";

window.onload = async () => {
  await WASM.default();
  window.WASM_FUNCTIONS = WASM;

  const mdInput = document.getElementById("md-input");
  const compiled = document.getElementById("compiled-text");
  mdInput.oninput = () => {
    const r = WASM.compile_md_from_js(mdInput.value);
    compiled.innerHTML = r;
  };
};
export {};
