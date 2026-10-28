async function runAgentReachCommand(
  args: readonly string[],
  environment: Readonly<Record<string, string | undefined>> = {},
): Promise<string> {
  const child = Bun.spawn([...args], {
    stdout: "pipe",
    stderr: "pipe",
    env: { ...process.env, ...environment },
    timeout: 180_000,
  });
  const [output, errors, status] = await Promise.all([
    new Response(child.stdout).text(),
    new Response(child.stderr).text(),
    child.exited,
  ]);
  if (status !== 0) {
    throw new Error(`${args[0]} failed (${status}): ${errors.trim()}`);
  }
  return output.trim();
}

export { runAgentReachCommand };
