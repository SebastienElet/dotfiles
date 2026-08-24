export class HandoffError extends Error {
  public readonly exitCode: number;

  public constructor(message: string, exitCode: number) {
    super(message);
    this.exitCode = exitCode;
    this.name = "HandoffError";
  }
}
