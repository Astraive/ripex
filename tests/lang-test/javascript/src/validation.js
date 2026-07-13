// ripex-lang-test: JS validation — regex, arrow predicates.
export function isEmail(s) {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(s);
}

export function validateUser(user) {
  if (!user?.name) throw new Error('name required');
  if (!isEmail(user.email)) throw new Error('bad email');
  return true;
}

export function sanitize(input) {
  return String(input).trim().toLowerCase();
}
