const commandPath =
  "$HOME/.local/bin:$HOME/.volta/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin";
const mcpServerArguments = [
  "--",
  "/bin/sh",
  "-c",
  `export PATH="${commandPath}"; exec "$HOME/.local/bin/remem" mcp`,
] as const;

function succeeds(command: readonly string[]): boolean {
  return (
    Bun.spawnSync([...command], { stdout: "ignore", stderr: "ignore" })
      .exitCode === 0
  );
}

function run(
  command: readonly string[],
  options: Readonly<{ discardStdout: boolean }>,
): void {
  const result = Bun.spawnSync([...command], {
    stdout: options.discardStdout ? "ignore" : "inherit",
    stderr: "inherit",
  });
  if (result.exitCode !== 0) {
    throw new Error(
      `${command.join(" ")} exited with status ${result.exitCode}`,
    );
  }
}

export { mcpServerArguments, run, succeeds };
