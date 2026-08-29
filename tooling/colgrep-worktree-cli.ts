#!/usr/bin/env bun

import { main } from "./colgrep-worktree.ts";

process.exitCode = main();
