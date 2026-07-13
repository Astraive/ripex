// ripex-lang-test: JS string utils — template literals, optional chaining, methods.
export function greet(name) {
  return `Hello, ${name}!`;
}

export const capitalize = (s) => s.charAt(0).toUpperCase() + s.slice(1);

export function maskEmail(email) {
  const [user, domain] = email.split('@');
  return `${user[0]}***@${domain}`;
}

export const truncate = (s, n = 10) => (s.length > n ? s.slice(0, n) + '…' : s);

export function describeUser(user) {
  return `${user?.name ?? 'anon'} (${user?.profile?.age ?? 0})`;
}

export function compose(a, b) {
  return (x) => a(b(x));
}
