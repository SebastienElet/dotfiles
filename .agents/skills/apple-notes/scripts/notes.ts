#!/usr/bin/env bun

import { runNotesCommand } from "./notes-command.ts";

const argumentOffset = 2;
const commandArguments = process.argv.slice(argumentOffset);
const body = await Bun.stdin.text();
const exitCode = await runNotesCommand(commandArguments, body);

process.exit(exitCode);
