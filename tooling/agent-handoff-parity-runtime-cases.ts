import {
  type ParityCase,
  environmentFor,
  event,
  parityCase,
  prepareClaude,
  sentinelPath,
} from "./agent-handoff-parity-fixtures.ts";
import { dirname, join } from "node:path";
import { mkdirSync, symlinkSync, writeFileSync } from "node:fs";

const invalidUtf8Byte = 0xff;
const textEncoder = new TextEncoder();

function bytesAroundInvalidUtf8(prefix: string, suffix: string): Uint8Array {
  return Uint8Array.from([
    ...textEncoder.encode(prefix),
    invalidUtf8Byte,
    ...textEncoder.encode(suffix),
  ]);
}

const runtimeParityCases: ParityCase[] = [
  parityCase("isolated invalid UTF-8 byte in transcript", event, {
    prepare: (fixture) => {
      writeFileSync(fixture.transcriptPath, Uint8Array.from([invalidUtf8Byte]));
    },
  }),
  parityCase("invalid UTF-8 byte inside ignored transcript text", event, {
    prepare: (fixture) => {
      writeFileSync(
        fixture.transcriptPath,
        bytesAroundInvalidUtf8(
          '{"metadata":"',
          '","type":"assistant","isSidechain":false,"message":{"usage":{"cache_creation_input_tokens":0,"cache_read_input_tokens":0,"input_tokens":90000}}}\n',
        ),
      );
    },
  }),
  parityCase(
    "XDG state root lexically removes a non-directory parent component",
    event,
    {
      environment: (fixture) =>
        environmentFor(fixture, {
          xdgStateHome: `${fixture.xdgStateHome}/file/..`,
        }),
      prepare: (fixture) => {
        prepareClaude()(fixture);
        mkdirSync(fixture.xdgStateHome, { recursive: true });
        writeFileSync(join(fixture.xdgStateHome, "file"), "");
      },
    },
  ),
  parityCase(
    "XDG state root lexically removes a symlink parent component",
    event,
    {
      environment: (fixture) =>
        environmentFor(fixture, {
          xdgStateHome: `${fixture.xdgStateHome}/alias/..`,
        }),
      prepare: (fixture) => {
        prepareClaude()(fixture);
        mkdirSync(fixture.xdgStateHome, { recursive: true });
        const target = join(fixture.root, "target");
        mkdirSync(target);
        symlinkSync(target, join(fixture.xdgStateHome, "alias"), "dir");
        const sentinel = sentinelPath(fixture, "session");
        mkdirSync(dirname(sentinel), { recursive: true });
        writeFileSync(sentinel, "");
      },
    },
  ),
];

export { runtimeParityCases };
