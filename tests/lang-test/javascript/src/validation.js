// ripex-lang-test: JS validation — regex, arrow predicates.
/** @param {string} s */
export function isEmail(s) {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(s);
}

/** @param {{ name?: string, email: string } | null | undefined} user */
export function validateUser(user) {
  if (!user?.name) throw new Error('name required');
  if (!isEmail(user.email)) throw new Error('bad email');
  return true;
}

/** @param {unknown} input */
export function sanitize(input) {
  return String(input).trim().toLowerCase();
}
