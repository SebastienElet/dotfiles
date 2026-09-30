import { mkdirSync, renameSync, writeFileSync } from "node:fs";
import { run, succeeds } from "./remem-process.ts";
import { join } from "node:path";
import { z } from "zod";

const argumentsSchema = z.tuple([z.string().min(1)]);
const label = "dev.remem.worker";
const argumentOffset = 2;

type PlistValue =
  | string
  | number
  | boolean
  | readonly PlistValue[]
  | { readonly [key: string]: PlistValue };

function escapeMarkup(text: string): string {
  return text
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
}

function renderPlist(value: PlistValue, depth: number): string {
  const indent = "\t".repeat(depth);
  if (typeof value === "string") {
    return `${indent}<string>${escapeMarkup(value)}</string>`;
  }
  if (typeof value === "number") {
    return `${indent}<integer>${value}</integer>`;
  }
  if (typeof value === "boolean") {
    return `${indent}<${value}/>`;
  }
  if (Array.isArray(value)) {
    const items: readonly PlistValue[] = value;
    return [
      `${indent}<array>`,
      ...items.map((item) => renderPlist(item, depth + 1)),
      `${indent}</array>`,
    ].join("\n");
  }
  return [
    `${indent}<dict>`,
    ...Object.entries(value).flatMap(
      ([key, entry]: readonly [string, PlistValue]) => [
        `${indent}\t<key>${escapeMarkup(key)}</key>`,
        renderPlist(entry, depth + 1),
      ],
    ),
    `${indent}</dict>`,
  ].join("\n");
}

function workerDefinition(home: string): string {
  const definition = {
    Label: label,
    ProgramArguments: [
      join(home, ".local", "bin", "remem"),
      "worker",
      "--once",
    ],
    EnvironmentVariables: {
      PATH: `${home}/.local/bin:${home}/.volta/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin`,
    },
    RunAtLoad: true,
    StartInterval: 300,
    ProcessType: "Background",
  };
  return `<?xml version="1.0" encoding="UTF-8"?>\n<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">\n<plist version="1.0">\n${renderPlist(definition, 0)}\n</plist>\n`;
}

function install(home: string): void {
  const directory = join(home, "Library", "LaunchAgents");
  const path = join(directory, `${label}.plist`);
  const temporary = `${path}.${process.pid}.tmp`;
  mkdirSync(directory, { recursive: true });
  writeFileSync(temporary, workerDefinition(home));
  renameSync(temporary, path);
  const domain = `gui/${process.getuid?.() ?? ""}`;
  if (succeeds(["launchctl", "print", `${domain}/${label}`])) {
    run(["launchctl", "bootout", `${domain}/${label}`], {
      discardStdout: false,
    });
  }
  run(["launchctl", "bootstrap", domain, path], { discardStdout: false });
}

if (import.meta.main) {
  try {
    const [home] = argumentsSchema.parse(process.argv.slice(argumentOffset));
    install(home);
  } catch (error) {
    process.stderr.write(
      `Error: ${error instanceof Error ? error.message : String(error)}\n`,
    );
    process.exitCode = 1;
  }
}
