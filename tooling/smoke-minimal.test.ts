import { expect, test } from "bun:test";
import type { ProfileOperations } from "./smoke-minimal.ts";
import { smokeMinimalProfile } from "./smoke-minimal.ts";

function profile(
  failure?: "clean" | "restore",
): Readonly<{ operations: ProfileOperations; restored: () => boolean }> {
  let installed = false;
  let cleaned = false;
  let restored = false;
  const output = { stdout: Buffer.from(""), stderr: Buffer.from("") };
  return {
    operations: {
      install: () => {
        if (cleaned && !installed) {
          if (failure === "restore") {
            throw new Error("restoration failed");
          }
          restored = true;
        }
        installed = true;
        return output;
      },
      clean: () => {
        if (!installed) {
          throw new Error("cleanup requires an installed profile");
        }
        if (failure === "clean") {
          throw new Error("cleanup failed");
        }
        installed = false;
        cleaned = true;
        return output;
      },
      verify: () => {
        if (!installed) {
          throw new Error("profile is absent");
        }
      },
      snapshot: () => {
        if (!installed) {
          throw new Error("profile is absent");
        }
        return "installed identity";
      },
    },
    restored: () => restored,
  };
}

test("restores the profile after successful cleanup and verifies its replay", () => {
  const context = profile();
  smokeMinimalProfile(context.operations);
  expect(context.restored()).toBeTrue();
});

test.each(["clean", "restore"] as const)(
  "propagates %s failure without reporting restoration",
  (failure) => {
    const context = profile(failure);
    expect(() => {
      smokeMinimalProfile(context.operations);
    }).toThrow();
    expect(context.restored()).toBeFalse();
  },
);
