import { isAbsolute } from "node:path";

export const validateDefaultCorpus = (userInstructions: string): string[] => {
  const matches = [
    ...userInstructions.matchAll(
      /^- \*\*Default Obsidian corpus:\*\* `(?<corpus>[^`\n]+)`$/gmu,
    ),
  ];
  if (matches.length !== 1) {
    return ["USER.md must declare exactly one default Obsidian corpus"];
  }
  const defaultCorpus = matches[0]?.groups?.corpus;
  return defaultCorpus !== undefined && isAbsolute(defaultCorpus)
    ? []
    : ["the default Obsidian corpus must be an absolute path"];
};
