import { dirname, join } from "node:path";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { z } from "zod";

const home = dirname(dirname(dirname(process.argv[1] ?? "")));
const marker = join(home, "worker-native-state.json");
const input = z
  .object({ plist: z.string().min(1) })
  .parse(
    JSON.parse(readFileSync(join(home, "worker-native-input.json"), "utf8")),
  );
process.on("SIGTERM", (): never => {
  writeFileSync(
    marker,
    JSON.stringify({
      pid: process.pid,
      stopped: true,
      configPresent: existsSync(join(home, ".remem/config.toml")),
      plistPresent: existsSync(input.plist),
    }),
  );
  process.exit(0);
});
writeFileSync(marker, JSON.stringify({ pid: process.pid, stopped: false }));
const lifetimeMilliseconds = 60_000;
await Bun.sleep(lifetimeMilliseconds);
