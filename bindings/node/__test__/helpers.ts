import { fileURLToPath } from "node:url";

/** The repository root, four levels up from `bindings/node/__test__`. */
export const REPO = fileURLToPath(new URL("../../..", import.meta.url));
export const FIXTURES = `${REPO}fixtures`;
export const FONTS = `${FIXTURES}/font`;

export const PNG_MAGIC = Uint8Array.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);

export function startsWith(bytes: Uint8Array, prefix: Uint8Array | string): boolean {
  const expected =
    typeof prefix === "string"
      ? Uint8Array.from(prefix, (c) => c.charCodeAt(0))
      : prefix;
  return expected.every((byte, index) => bytes[index] === byte);
}
