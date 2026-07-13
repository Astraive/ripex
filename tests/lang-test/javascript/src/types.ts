// ripex-lang-test: TypeScript fixtures — interfaces, generics, enums, decorators, type aliases.
export interface Identifiable {
  id: number;
  readonly createdAt: Date;
}

export type Result<T> = { ok: true; value: T } | { ok: false; error: string };

export enum Status {
  Active = 'active',
  Inactive = 'inactive',
  Pending = 'pending',
}

export function firstOrNull<T>(xs: T[]): T | null {
  return xs.length > 0 ? xs[0] : null;
}

export class Repository<T extends Identifiable> {
  private items: Map<number, T> = new Map();

  add(item: T): void {
    this.items.set(item.id, item);
  }

  get(id: number): T | undefined {
    return this.items.get(id);
  }

  @loggedMethod
  clear(): void {
    this.items.clear();
  }
}

function loggedMethod(_target: unknown, _ctx: unknown) {
  return _target;
}
