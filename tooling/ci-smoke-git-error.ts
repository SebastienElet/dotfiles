export class GitError extends Error {
  public readonly details: string;

  public constructor(details: string) {
    super("Git command failed");
    this.details = details;
    this.name = "GitError";
  }
}
