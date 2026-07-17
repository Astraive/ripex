// ripex-lang-test: JS class model — decorators, static block, methods, getters/setters.
export class User {
  #password = '';

  static INSTANCE_COUNT = 0;

  static ROLE = 'user';

  static {
    User.INSTANCE_COUNT = 0;
  }

  /** @param {string} name @param {string} email @param {string} role */
  constructor(name, email, role = 'user') {
    this.name = name;
    this.email = email;
    this.role = role;
  }

  get password() {
    return this.#password;
  }

  set password(p) {
    this.#password = p;
  }

  describe() {
    return `${this.name} <${this.email}>`;
  }

  get isAdmin() {
    return this.role === 'admin';
  }
}

export class AdminUser extends User {
  /** @param {string} name @param {string} email */
  constructor(name, email) {
    super(name, email, 'admin');
  }

  describe() {
    return `ADMIN ${super.describe()}`;
  }
}
