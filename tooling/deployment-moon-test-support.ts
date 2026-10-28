function withoutMoonTaskContext(
  environment: Readonly<NodeJS.ProcessEnv>,
): NodeJS.ProcessEnv {
  return Object.fromEntries(
    Object.entries(environment).filter(
      (entry: readonly [string, string | undefined]) =>
        !entry[0].startsWith("MOON_"),
    ),
  );
}

export { withoutMoonTaskContext };
