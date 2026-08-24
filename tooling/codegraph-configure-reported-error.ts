class ReportedCommandError extends Error {
  public override readonly name = "ReportedCommandError";

  public constructor(public readonly exitCode: number) {
    super(`command failed with exit ${exitCode}`);
  }
}

export { ReportedCommandError };
