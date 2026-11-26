import init, { adjust, merge_spectra_json } from "/images/lightmatter.js";

let wasmReady;

self.addEventListener("message", async (event) => {
  const { id, op, text, options, parts } = event.data;
  try {
    wasmReady ||= init();
    await wasmReady;
    if (op === "merge") {
      self.postMessage({ id, result: merge_spectra_json(JSON.stringify(parts)) });
      return;
    }
    self.postMessage({ id, result: adjust(text, JSON.stringify(options)) });
  } catch (error) {
    self.postMessage({ id, error: String(error) });
  }
});