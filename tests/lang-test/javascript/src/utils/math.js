// ripex-lang-test: JS math utilities — exercises arrow fns, defaults, rest, throws.
export const add = (a, b) => a + b;

export const subtract = (a, b = 1) => a - b;

export function multiply(a, b = 1) {
  return a * b;
}

export const divide = (a, b = 1) => {
  if (b === 0) throw new Error('Division by zero');
  return a / b;
};

export const max = (...numbers) =>
  numbers.reduce((a, b) => (a > b ? a : b), -Infinity);

export const clamp = (n, lo, hi) => Math.min(Math.max(n, lo), hi);

function privateHelper(x) {
  return x * 2;
}

export function usesPrivate(y) {
  return privateHelper(y) + 1;
}
