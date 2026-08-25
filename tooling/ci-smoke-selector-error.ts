export class SelectorError extends Error {
  public readonly details: string;

  public constructor(message: string, details = "") {
    super(message);
    this.details = details;
    this.name = "SelectorError";
  }
}
