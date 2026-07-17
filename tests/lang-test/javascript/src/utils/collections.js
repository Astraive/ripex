// ripex-lang-test: JS collections — higher-order fns, generators, async.
/** @param {number[]} xs */
export const doubleAll = (xs) => xs.map((x) => x * 2);

/** @param {number[]} xs */
export function filterPositive(xs) {
  return xs.filter((x) => x > 0);
}

/** @param {number} start @param {number} end @returns {Generator<number>} */
export function* range(start, end) {
  for (let i = start; i < end; i++) yield i;
}

/** @param {number[]} xs */
export const sumArray = (xs) => xs.reduce((acc, x) => acc + x, 0);

/** @template T @param {T[]} xs @param {number} count */
export const take = (xs, count) => xs.slice(0, count);

/** @returns {Generator<number>} */
export function* idGenerator() {
  let id = 0;
  while (true) yield id++;
}

/** @param {number[]} xs @param {number} min @param {number} max */
export const filterInRange = (xs, min, max) =>
  xs.filter((value) => value >= min && value <= max);

/** @param {string | URL} url @returns {Promise<unknown>} */
export async function fetchJson(url) {
  const res = await fetch(url);
  return res.json();
}
