import { quoteAppleScript } from "./notes-applescript.ts";

function optionalArgument(
  arguments_: readonly string[],
  index: number,
): string | undefined {
  const value = arguments_.at(index);
  return typeof value === "string" ? value : undefined;
}

function optionalNonemptyArgument(
  arguments_: readonly string[],
  index: number,
): string | undefined {
  const value = optionalArgument(arguments_, index);
  return value === "" ? undefined : value;
}

function requiredArgument(
  arguments_: readonly string[],
  index: number,
  usage: string,
): string {
  const value = optionalArgument(arguments_, index);
  if (value === undefined || value === "") {
    throw new Error(usage);
  }
  return value;
}

function selectedAccount(arguments_: readonly string[], index: number): string {
  return quoteAppleScript(optionalArgument(arguments_, index) ?? "iCloud");
}

export {
  optionalArgument,
  optionalNonemptyArgument,
  requiredArgument,
  selectedAccount,
};
