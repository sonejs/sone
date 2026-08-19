/**
 * Reading bytes, and the platform's preferred byte type.
 *
 * Lifted from sone v2's `backend.ts`, which had exactly this problem: one
 * package, several runtimes, and only the way raw bytes are read genuinely
 * differs between them.
 */
import type { FontSource } from "./config.ts";

const proc = (globalThis as { process?: { versions?: Record<string, string> } })
  .process;

/** Node, Bun and Deno all report a `node` version and all have `node:fs`. */
export const hasNodeFs = proc?.versions?.node != null;

export const isHttp = (src: string): boolean => /^https?:\/\//.test(src);

export const isDataUrl = (src: string): boolean => /^data:/.test(src);

async function fetchBytes(src: string | URL): Promise<Uint8Array> {
  const response = await fetch(src);
  if (!response.ok) {
    throw new Error(`Failed to fetch ${String(src)} (HTTP ${response.status})`);
  }
  return new Uint8Array(await response.arrayBuffer());
}

async function readFileBytes(src: string | URL): Promise<Uint8Array> {
  const { readFile } = await import(/* @vite-ignore */ "node:fs/promises");
  return new Uint8Array(await readFile(src));
}

/** Read bytes from a file path (Node/Bun/Deno) or an `http(s)` URL string. */
export function readBytes(src: string): Promise<Uint8Array> {
  if (isHttp(src)) return fetchBytes(src);
  if (hasNodeFs) return readFileBytes(src);
  // In the browser a bare string is a URL relative to the document.
  return fetchBytes(src);
}

/** Read bytes from a `URL`, honouring `file:` where there is a filesystem. */
export function readUrl(url: URL): Promise<Uint8Array> {
  if (url.protocol === "file:") {
    if (hasNodeFs) return readFileBytes(url);
    throw new Error("file: URLs are only supported where there is a filesystem");
  }
  return fetchBytes(url);
}

/** Resolve any accepted font source to raw bytes. */
export async function loadFontBytes(source: FontSource): Promise<Uint8Array[]> {
  if (source instanceof Uint8Array) return [source];
  if (source instanceof ArrayBuffer) return [new Uint8Array(source)];
  if (source instanceof URL) return [await readUrl(source)];
  if (Array.isArray(source)) {
    const all = await Promise.all(source.map((entry) => loadFontBytes(entry)));
    return all.flat();
  }
  return [await readBytes(source)];
}

/** Node preserves its historical `Buffer` output; other runtimes get bytes. */
const NodeBuffer = (
  globalThis as { Buffer?: { from(input: Uint8Array): Uint8Array } }
).Buffer;

/** Wrap encoded output in the platform's preferred byte type. */
export const wrap = (bytes: Uint8Array): Uint8Array =>
  hasNodeFs && NodeBuffer != null ? NodeBuffer.from(bytes) : bytes;
