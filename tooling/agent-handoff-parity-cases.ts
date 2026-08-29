import {
  type ClaudeUsageOptions,
  type ParityCase,
  belowThresholdUsage,
  bytes,
  claudeUsage,
  codexUsage,
  defaultWindow,
  environmentFor,
  event,
  fractionalTokenCount,
  highUsage,
  invalidSessionCase,
  invalidUtf8Byte,
  lowUsage,
  parityCase,
  prepareClaude,
  prepareTranscript,
  retainedLineCounts,
  retainedWindowLineCount,
  sentinelPath,
  thresholdUsage,
} from "./agent-handoff-parity-fixtures.ts";
import {
  type Fixture,
  setFixtureCommandArguments,
} from "./agent-handoff-parity-support.ts";
import { dirname, join } from "node:path";
import { mkdirSync, writeFileSync } from "node:fs";

const basicInputCases: readonly (readonly [string, ParityCase["input"]])[] = [
  ["invalid JSON", (): Uint8Array => bytes("not-json")],
  ["JSON null", (): Uint8Array => bytes("null")],
  [
    "object without an event",
    (fixture: Fixture): Uint8Array =>
      bytes(
        JSON.stringify({
          session_id: "session",
          transcript_path: fixture.transcriptPath,
        }),
      ),
  ],
  [
    "Claude event other than Stop",
    (fixture: Fixture): Uint8Array =>
      event(fixture, { hook_event_name: "UserPromptSubmit" }),
  ],
  [
    "Codex event other than Stop",
    (fixture: Fixture): Uint8Array =>
      event(fixture, { event: "UserPromptSubmit", hook_event_name: undefined }),
  ],
  [
    "two event names with one contradictory",
    (fixture: Fixture): Uint8Array =>
      event(fixture, { event: "UserPromptSubmit" }),
  ],
];

const sessionCases = [
  invalidSessionCase("missing session_id"),
  invalidSessionCase("empty session_id", ""),
  invalidSessionCase("session_id equal to dot", "."),
  invalidSessionCase("session_id equal to dot dot", ".."),
  invalidSessionCase("session_id containing slash", "a/b"),
  parityCase(
    "session_id with letters digits dot underscore and hyphen",
    (fixture) => event(fixture, { session_id: "Ab9._-" }),
    { prepare: prepareClaude() },
  ),
];

const invalidTokenCases: readonly (readonly [
  string,
  number,
  ClaudeUsageOptions,
])[] = [
  ["negative token number", -1, {}],
  ["fractional token number", fractionalTokenCount, {}],
  [
    "token number above Number.MAX_SAFE_INTEGER",
    Number.MAX_SAFE_INTEGER + 1,
    {},
  ],
  [
    "Claude token total above Number.MAX_SAFE_INTEGER",
    Number.MAX_SAFE_INTEGER,
    { cacheReadInputTokens: 1 },
  ],
];

const claudeTranscriptCases: readonly (readonly [string, number])[] = [
  ["Claude transcript below threshold", belowThresholdUsage],
  ["Claude transcript at 85 percent", thresholdUsage],
  ["Claude transcript above threshold", highUsage],
];

const thresholdCases = [
  ["valid explicit threshold", "50000"],
  ["empty explicit threshold", ""],
  ["zero explicit threshold", "0"],
  ["negative explicit threshold", "-1"],
  ["fractional explicit threshold", "1.5"],
  ["85k explicit threshold", "85k"],
  [
    "explicit threshold above Number.MAX_SAFE_INTEGER",
    String(BigInt(Number.MAX_SAFE_INTEGER) + 1n),
  ],
] as const;

const parityCases: ParityCase[] = [
  ...basicInputCases.map(([name, input]) => parityCase(name, input)),
  ...sessionCases,
  parityCase("missing transcript_path", (fixture) =>
    event(fixture, { transcript_path: undefined }),
  ),
  parityCase("absent transcript file", event),
  parityCase("non-boolean stop_hook_active", (fixture) =>
    event(fixture, { stop_hook_active: "true" }),
  ),
  parityCase(
    "active hook with absent transcript and environment",
    (fixture) => event(fixture, { stop_hook_active: true }),
    {
      environment: (fixture) =>
        environmentFor(fixture, {
          claudeWindow: false,
          home: false,
          xdgStateHome: false,
        }),
    },
  ),
  ...claudeTranscriptCases.map(([name, used]) =>
    parityCase(name, event, { prepare: prepareClaude(used) }),
  ),
  parityCase("Claude sidechain transcript", event, {
    prepare: prepareTranscript([
      claudeUsage(lowUsage),
      claudeUsage(highUsage, { sidechain: true }),
    ]),
  }),
  parityCase("Claude transcript uses latest main-chain record", event, {
    prepare: prepareTranscript([
      claudeUsage(lowUsage),
      claudeUsage(highUsage, { sidechain: true }),
      claudeUsage(highUsage),
    ]),
  }),
  parityCase(
    "Codex transcript uses its window and dollar handoff invocation",
    (fixture) => event(fixture, { event: "Stop", hook_event_name: undefined }),
    { prepare: prepareTranscript([codexUsage(highUsage, defaultWindow)]) },
  ),
  ...retainedLineCounts.map((physicalLineCount) =>
    parityCase(`${physicalLineCount} latest physical transcript lines`, event, {
      prepare: prepareTranscript([
        claudeUsage(highUsage),
        ...Array.from({ length: physicalLineCount - 1 }, () => ""),
      ]),
    }),
  ),
  parityCase("malformed JSON line retained in transcript window", event, {
    prepare: prepareTranscript([claudeUsage(highUsage), "not-json"]),
  }),
  parityCase("malformed JSON line outside transcript window", event, {
    prepare: prepareTranscript([
      "not-json",
      ...Array.from({ length: retainedWindowLineCount - 1 }, () => ""),
      claudeUsage(highUsage),
    ]),
  }),
  ...invalidTokenCases.map(([name, inputTokens, options]) =>
    parityCase(name, event, {
      prepare: prepareTranscript([claudeUsage(inputTokens, options)]),
    }),
  ),
  parityCase(
    "zero Codex context window",
    (fixture) => event(fixture, { event: "Stop", hook_event_name: undefined }),
    { prepare: prepareTranscript([codexUsage(highUsage, 0)]) },
  ),
  ...thresholdCases.map(([name, threshold]) =>
    parityCase(name, event, {
      environment: (fixture) =>
        environmentFor(fixture, {
          overrides: { HANDOFF_TOKEN_THRESHOLD: threshold },
        }),
      prepare: prepareClaude(),
    }),
  ),
  parityCase("absent Claude context window", event, {
    environment: (fixture) => environmentFor(fixture, { claudeWindow: false }),
    prepare: prepareClaude(),
  }),
  parityCase("empty Claude context window", event, {
    environment: (fixture) => environmentFor(fixture, { claudeWindow: "" }),
    prepare: prepareClaude(),
  }),
  parityCase("valid Claude context window", event, {
    environment: (fixture) =>
      environmentFor(fixture, { claudeWindow: "100001" }),
    prepare: prepareClaude(thresholdUsage),
  }),
  parityCase(
    "XDG_STATE_HOME priority over HOME",
    (fixture) => event(fixture, { session_id: "xdg-priority" }),
    {
      prepare: (fixture) => {
        prepareClaude()(fixture);
        const sentinel = join(
          fixture.home,
          ".local/state/dotfiles/handoff/xdg-priority",
        );
        mkdirSync(dirname(sentinel), { recursive: true });
        writeFileSync(sentinel, "");
      },
    },
  ),
  parityCase("HOME fallback", event, {
    environment: (fixture) => environmentFor(fixture, { xdgStateHome: "" }),
    prepare: prepareClaude(),
  }),
  parityCase("absence of HOME and XDG_STATE_HOME", event, {
    environment: (fixture) =>
      environmentFor(fixture, { home: false, xdgStateHome: false }),
    prepare: prepareClaude(),
  }),
  parityCase(
    "sentinel file already present",
    (fixture) => event(fixture, { session_id: "sentinel-file" }),
    {
      prepare: (fixture) => {
        prepareClaude()(fixture);
        const sentinel = sentinelPath(fixture, "sentinel-file");
        mkdirSync(dirname(sentinel), { recursive: true });
        writeFileSync(sentinel, "");
      },
    },
  ),
  parityCase(
    "sentinel directory already present",
    (fixture) => event(fixture, { session_id: "sentinel-directory" }),
    {
      prepare: (fixture) => {
        prepareClaude()(fixture);
        mkdirSync(sentinelPath(fixture, "sentinel-directory"), {
          recursive: true,
        });
      },
    },
  ),
  parityCase("sentinel parent is a file instead of a directory", event, {
    prepare: (fixture) => {
      prepareClaude()(fixture);
      const parent = dirname(sentinelPath(fixture, "session"));
      mkdirSync(dirname(parent), { recursive: true });
      writeFileSync(parent, "");
    },
  }),
  parityCase("non-UTF-8 stdin", () => Uint8Array.from([invalidUtf8Byte])),
  parityCase("extra CLI argument ignored", event, {
    prepare: (fixture) => {
      setFixtureCommandArguments(fixture, ["ignored"]);
      prepareClaude()(fixture);
    },
  }),
];

const concurrentParityCase = parityCase(
  "three concurrent processes of the same session_id",
  (fixture) => event(fixture, { session_id: "concurrent" }),
  { prepare: prepareClaude() },
);

export { concurrentParityCase, parityCases };
