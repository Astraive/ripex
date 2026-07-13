// ripex-lang-test: JS entry — bushy import graph + re-export namespace (gap).
import { add, subtract, multiply, divide, power, max } from './utils/math.js';
import { greet, capitalize, pluralize, truncate } from './utils/strings.js';
import { User } from './models/user.js';
import { Product } from './models/product.js';
import { validateUser, isEmail, sanitize } from './utils/validation.js';
import { calculateArea, safeJsonParse } from './async.js';
import {
  doubleAll,
  filterPositive,
  sumArray,
  range,
  take,
  idGenerator,
  filterInRange,
} from './utils/collections.js';

// Re-export namespace with alias (gap: export * as ns).
export * as mathUtils from './utils/math.js';
export * as models from './models/user.js';

export function runDemo() {
  const results = [];

  results.push(`math: add(5,3)=${add(5, 3)}, multiply(4)=${multiply(4)}, max(1,9,3)=${max(1, 9, 3)}`);

  const alice = new User('Alice', 'alice@example.com', 'admin');
  alice.password = 'secret123';
  results.push(alice.describe());
  results.push(`isAdmin: ${alice.isAdmin}`);

  const widget = new Product(1, 'Widget', 19.99, 'electronics');
  results.push(`tax: ${widget.calculateTax()}`);

  results.push(greet(alice.name));
  results.push(`email? ${isEmail(alice.email)}`);
  results.push(`pos: ${filterPositive([-1, 0, 2, -3, 4])}`);
  results.push(`sum: ${sumArray([1, 2, 3, 4])}`);

  return results;
}
