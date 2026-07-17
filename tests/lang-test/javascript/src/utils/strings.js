// ripex-lang-test: JS string utils — template literals, optional chaining, methods.
/** @param {string} name */
export function greet(name) {
  return `Hello, ${name}!`;
}

/** @param {string} s */
export const capitalize = (s) => s.charAt(0).toUpperCase() + s.slice(1);

/** @param {string} email */
export function maskEmail(email) {
  const [user, domain] = email.split('@');
  return `${user[0]}***@${domain}`;
}

/** @param {string} s @param {number} n */
export const truncate = (s, n = 10) => (s.length > n ? s.slice(0, n) + '…' : s);

/** @param {string} word @param {number} count */
export const pluralize = (word, count) => (count === 1 ? word : `${word}s`);

/** @param {{ name?: string, profile?: { age?: number } } | null} user */
export function describeUser(user) {
  return `${user?.name ?? 'anon'} (${user?.profile?.age ?? 0})`;
}

/**
 * @param {(value: number) => number} a
 * @param {(value: number) => number} b
 * @returns {(value: number) => number}
 */
export function compose(a, b) {
  return (x) => a(b(x));
}
