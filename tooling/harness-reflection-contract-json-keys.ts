type JsonString = Readonly<{
  nextIndex: number;
  value: string;
}>;

class JsonDuplicateKeyScanner {
  private readonly duplicates: string[] = [];
  private index = 0;

  public constructor(private readonly source: string) {}

  public scan(): readonly string[] {
    try {
      this.scanValue();
    } catch {
      return [];
    }
    return this.duplicates;
  }

  private skipWhitespace(): void {
    while (/\s/u.test(this.source[this.index] ?? "")) {
      this.index += 1;
    }
  }

  private readString(): JsonString {
    const start = this.index;
    let escaped = false;
    for (let index = start + 1; index < this.source.length; index += 1) {
      const character = this.source[index];
      if (escaped) {
        escaped = false;
      } else if (character === "\\") {
        escaped = true;
      } else if (character === '"') {
        const value: unknown = JSON.parse(this.source.slice(start, index + 1));
        if (typeof value !== "string") {
          throw new TypeError("JSON object key is not a string");
        }
        return { nextIndex: index + 1, value };
      }
    }
    throw new Error("unterminated JSON string");
  }

  private scanPrimitive(): void {
    while (
      this.index < this.source.length &&
      !/[\s,\]}]/u.test(this.source[this.index] ?? "")
    ) {
      this.index += 1;
    }
  }

  private scanValue(): void {
    this.skipWhitespace();
    const character = this.source[this.index];
    if (character === "{") {
      this.scanObject();
    } else if (character === "[") {
      this.scanArray();
    } else if (character === '"') {
      this.index = this.readString().nextIndex;
    } else {
      this.scanPrimitive();
    }
  }

  private scanArray(): void {
    this.index += 1;
    while (this.index < this.source.length) {
      this.skipWhitespace();
      if (this.source[this.index] === "]") {
        this.index += 1;
        return;
      }
      this.scanValue();
      this.skipWhitespace();
      if (this.source[this.index] === ",") {
        this.index += 1;
      }
    }
  }

  private scanObject(): void {
    const keys = new Set<string>();
    this.index += 1;
    while (this.index < this.source.length) {
      this.skipWhitespace();
      if (this.source[this.index] === "}") {
        this.index += 1;
        return;
      }
      const key = this.readString();
      if (keys.has(key.value)) {
        this.duplicates.push(key.value);
      }
      keys.add(key.value);
      this.index = key.nextIndex;
      this.skipWhitespace();
      if (this.source[this.index] !== ":") {
        throw new Error("missing JSON object colon");
      }
      this.index += 1;
      this.scanValue();
      this.skipWhitespace();
      if (this.source[this.index] === ",") {
        this.index += 1;
      }
    }
  }
}

const duplicateJsonObjectKeys = (source: string): readonly string[] =>
  new JsonDuplicateKeyScanner(source).scan();

export { duplicateJsonObjectKeys };
