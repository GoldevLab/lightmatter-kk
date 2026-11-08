import init, { adjust } from "/images/lightmatter.js";

let wasmReady;

self.addEventListener("message", async (event) => {
  const { id, text, options } = event.data;
  try {
    wasmReady ||= init();
    await wasmReady;
    self.postMessage({ id, result: adjust(text, JSON.stringify(options)) });
  } catch (error) {
    self.postMessage({ id, error: String(error) });
  }
});