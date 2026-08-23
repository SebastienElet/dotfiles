export function quoteAppleScript(value: string): string {
  return value.replaceAll("\\", "\\\\").replaceAll('"', '\\"');
}

export function folderSpecifier(path: string): string {
  const parts = path.split("/");
  if (parts.some((part) => part === ""))
    throw new Error(`invalid folder path: ${path}`);
  return parts.reduce(
    (specifier, part) =>
      specifier === ""
        ? `folder "${quoteAppleScript(part)}"`
        : `folder "${quoteAppleScript(part)}" of ${specifier}`,
    "",
  );
}

export function folderCreation(path: string): string {
  let specifier = "";
  return path
    .split("/")
    .map((part) => {
      const quoted = quoteAppleScript(part);
      const parent = specifier;
      specifier =
        parent === ""
          ? `folder "${quoted}"`
          : `folder "${quoted}" of ${parent}`;
      return parent === ""
        ? `if not (exists ${specifier}) then make new folder with properties {name:"${quoted}"}`
        : `if not (exists ${specifier}) then make new folder at ${parent} with properties {name:"${quoted}"}`;
    })
    .join("\n");
}

export function replaceTitle(body: string, title: string): string {
  const block = `<div><b><span style="font-size: 24px">${title}</span></b><br></div>`;
  const replaced = body.replace(/^\s*<div>.*?<\/div>/s, block);
  return replaced === body ? `${block}${body}` : replaced;
}
