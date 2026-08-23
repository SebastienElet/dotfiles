type Fixture = {
  repository?: boolean;
  localBranches?: string[];
  showRefStatus?: number;
  showRefStderr?: string;
  remotes?: Record<
    string,
    { urls: string[]; head?: string; headOutput?: string }
  >;
  contexts?: unknown;
  repositories?: Record<string, unknown>;
  failures?: Record<string, { status: number; stderr: string }>;
};

const fixture = (await Bun.file(
  process.env.FIXTURE_PATH ?? "",
).json()) as Fixture;
const command = process.argv[2];
const arguments_ = process.argv.slice(3);
const failure = fixture.failures?.[`${command} ${arguments_.join(" ")}`];

if (failure) {
  process.stderr.write(failure.stderr);
  process.exit(failure.status);
}

if (command === "git") {
  const operation = arguments_.findIndex((value) =>
    [
      "rev-parse",
      "show-ref",
      "remote",
      "ls-remote",
      "check-ref-format",
    ].includes(value),
  );
  const tail = arguments_.slice(operation);
  if (tail[0] === "rev-parse")
    process.exit(fixture.repository === false ? 128 : 0);
  if (tail[0] === "show-ref") {
    if (fixture.showRefStatus) {
      if (fixture.showRefStderr) process.stderr.write(fixture.showRefStderr);
      process.exit(fixture.showRefStatus);
    }
    const branch = tail.at(-1)?.replace("refs/heads/", "");
    process.exit(branch && fixture.localBranches?.includes(branch) ? 0 : 1);
  }
  if (tail[0] === "remote" && tail.length === 1) {
    process.stdout.write(`${Object.keys(fixture.remotes ?? {}).join("\n")}\n`);
    process.exit(0);
  }
  if (tail[0] === "remote" && tail[1] === "get-url") {
    const remote = fixture.remotes?.[tail.at(-1) ?? ""];
    if (!remote) process.exit(2);
    process.stdout.write(`${remote.urls.join("\n")}\n`);
    process.exit(0);
  }
  if (tail[0] === "ls-remote") {
    const remote = fixture.remotes?.[tail.at(-2) ?? ""];
    if (!remote) process.exit(2);
    process.stdout.write(
      remote.headOutput ??
        (remote.head
          ? `ref: refs/heads/${remote.head}\tHEAD\nabc\tHEAD\n`
          : ""),
    );
    process.exit(0);
  }
  if (tail[0] === "check-ref-format") {
    const branch = tail.at(-1);
    process.exit(
      branch && !branch.startsWith("-") && !branch.includes("..") ? 0 : 1,
    );
  }
}

if (command === "bkt" && arguments_.includes("context")) {
  process.stdout.write(
    JSON.stringify(
      fixture.contexts ?? {
        contexts: [{ name: "cloud", host: "api.bitbucket.org" }],
      },
    ),
  );
  process.exit(0);
}

if (command === "bkt" && arguments_.includes("api")) {
  if (arguments_[0] !== "--context" || arguments_[1] !== "cloud")
    process.exit(3);
  const identity = arguments_
    .find((value) => value.startsWith("/repositories/"))
    ?.replace("/repositories/", "");
  const response = fixture.repositories?.[identity ?? ""];
  if (!response) process.exit(2);
  process.stdout.write(JSON.stringify(response));
  process.exit(0);
}

process.exit(127);
