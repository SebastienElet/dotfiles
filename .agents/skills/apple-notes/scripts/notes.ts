#!/usr/bin/env bun

import { runNotesCommand } from "./notes-command.ts";

process.exit(
  await runNotesCommand(process.argv.slice(2), await Bun.stdin.text()),
);
