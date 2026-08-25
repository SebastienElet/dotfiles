function quoteAppleScript(value: string): string {
  return value
    .replaceAll("\\", String.raw`\\\\`)
    .replaceAll('"', String.raw`\"`);
}

function folderCreation(path: string): string {
  let specifier = "";
  return path
    .split("/")
    .map((part) => {
      const parent = specifier;
      const quoted = quoteAppleScript(part);
      if (parent === "") {
        specifier = `folder "${quoted}"`;
        return `if not (exists ${specifier}) then make new folder with properties {name:"${quoted}"}`;
      }
      specifier = `folder "${quoted}" of ${parent}`;
      return `if not (exists ${specifier}) then make new folder at ${parent} with properties {name:"${quoted}"}`;
    })
    .join("\n");
}

function folderSpecifier(path: string): string {
  const parts = path.split("/");
  if (parts.some((part) => part === "")) {
    throw new Error(`invalid folder path: ${path}`);
  }
  let specifier = "";
  for (const part of parts) {
    const folder = `folder "${quoteAppleScript(part)}"`;
    specifier = specifier === "" ? folder : `${folder} of ${specifier}`;
  }
  return specifier;
}

function replaceTitle(body: string, title: string): string {
  const block = `<div><b><span style="font-size: 24px">${title}</span></b><br></div>`;
  const replaced = body.replace(/^\s*<div>.*?<\/div>/su, block);
  if (replaced === body) {
    return `${block}${body}`;
  }
  return replaced;
}

export { folderCreation, folderSpecifier, quoteAppleScript, replaceTitle };
