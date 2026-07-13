// ripex-lang-test: JS class model — decorators, static block, methods, getters/setters.
function logged(target, context) {
  return target;
}

export class User {
  #password = '';

  @logged
  static ROLE = 'user';

  static {
    User.INSTANCE_COUNT = 0;
  }

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
  constructor(name, email) {
    super(name, email, 'admin');
  }

  describe() {
    return `ADMIN ${super.describe()}`;
  }
}
