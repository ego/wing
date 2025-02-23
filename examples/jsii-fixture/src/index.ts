export class JsiiClass {
  /** @internal */
  _field: number;

  constructor(value: number) {
    this._field = value;
  }

  public field(): number {
    return this._field;
  }

  public publicMethod(arg: string) {
    return `Got ${arg}`;
  }

  public applyClosure(num: number, closure: IFakeClosure): number {
    return closure(num);
  }

  protected protectedMethod(arg: string) {
    return `Got ${arg}`;
  }
}

/**
 * @callable
 */
export interface IFakeClosure {
  /** @internal */
  (x: number): number

  fn(x: number): number;
}