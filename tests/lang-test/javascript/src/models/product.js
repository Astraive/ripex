// ripex-lang-test: JS product model — enum-ish, methods, computed names.
export const Category = {
  Electronics: 'electronics',
  Clothing: 'clothing',
};

export class Product {
  /**
   * @param {number} id
   * @param {string} name
   * @param {number} price
   * @param {string} category
   */
  constructor(id, name, price, category) {
    this.id = id;
    this.name = name;
    this.price = price;
    this.category = category;
  }

  get ['displayName']() {
    return `${this.name} (${this.category})`;
  }

  /** @param {number} rate */
  calculateTax(rate = 0.1) {
    return this.price * rate;
  }
}
