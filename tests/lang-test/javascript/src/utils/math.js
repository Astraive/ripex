// ripex-lang-test: JS math utilities — exercises arrow fns, defaults, rest, throws.
/** @param {number} a @param {number} b */
export const add = (a, b) => a + b;

/** @param {number} a @param {number} b */
export const subtract = (a, b = 1) => a - b;

/** @param {number} a @param {number} b */
export function multiply(a, b = 1) {
  return a * b;
}

/** @param {number} a @param {number} b */
export const divide = (a, b = 1) => {
  if (b === 0) throw new Error('Division by zero');
  return a / b;
};

/** @param {number} base @param {number} exponent */
export const power = (base, exponent) => base ** exponent;

/** @param {number[]} numbers */
export const max = (...numbers) =>
  numbers.reduce((a, b) => (a > b ? a : b), -Infinity);

/** @param {number} n @param {number} lo @param {number} hi */
export const clamp = (n, lo, hi) => Math.min(Math.max(n, lo), hi);

/** @param {number} x */
function privateHelper(x) {
  return x * 2;
}

/** @param {number} y */
export function usesPrivate(y) {
  return privateHelper(y) + 1;
}
