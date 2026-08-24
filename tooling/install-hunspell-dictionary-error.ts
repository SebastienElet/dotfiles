export class DictionaryInstallationError extends Error {
  public readonly exitCode: number;

  public constructor(message: string, exitCode = 1) {
    super(message);
    this.exitCode = exitCode;
    this.name = "DictionaryInstallationError";
  }
}
