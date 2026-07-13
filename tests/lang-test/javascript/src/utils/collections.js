// ripex-lang-test: JS collections — higher-order fns, generators, async.
export const doubleAll = (xs) => xs.map((x) => x * 2);

export function filterPositive(xs) {
  return xs.filter((x) => x > 0);
}

export function* range(start, end) {
  for (let i = start; i < end; i++) yield i;
}

export const sumArray = (xs) => xs.reduce((acc, x) => acc + x, 0);

export async function fetchJson(url) {
  const res = await fetch(url);
  return res.json();
}
