// ripex-lang-test: JS async + top-level await + re-export namespace (gap feature).
export async function calculateArea(w, h) {
  const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
  await sleep(0);
  return w * h;
}

export async function safeJsonParse(text) {
  try {
    return JSON.parse(text);
  } catch (e) {
    return null;
  }
}

// TOP-LEVEL AWAIT (gap: parser may not treat module scope as async)
const config = await fetch('/config.json').then((r) => r.json());

export const CONFIG = config;
