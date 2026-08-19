/**
 * One error class per failure class, mapped from `SoneError::exit_code()` —
 * the rule every sone binding follows.
 *
 * Node-API gives a thrown error's `code` from its napi status, which cannot be
 * customized, so the engine prefixes its message with `sone:<class>:` and this
 * is where that prefix is turned back into a type and stripped.
 */

export class SoneError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "SoneError";
  }
}

/** The IR document could not be parsed. */
export class IrError extends SoneError {
  constructor(message: string) {
    super(message);
    this.name = "IrError";
  }
}

/** A font or image could not be loaded. */
export class AssetError extends SoneError {
  constructor(message: string) {
    super(message);
    this.name = "AssetError";
  }
}

/** Layout or rasterization failed. */
export class RenderError extends SoneError {
  constructor(message: string) {
    super(message);
    this.name = "RenderError";
  }
}

const PREFIX = /^sone:(ir|asset|render):\s*/;

const BY_CLASS = {
  ir: IrError,
  asset: AssetError,
  render: RenderError,
} as const;

/** Rethrow an engine failure as the matching class. Anything else passes through. */
export function toSoneError(error: unknown): unknown {
  if (!(error instanceof Error)) return error;
  const match = PREFIX.exec(error.message);
  if (match == null) return error;
  const typed = new BY_CLASS[match[1] as keyof typeof BY_CLASS](
    error.message.slice(match[0].length),
  );
  typed.stack = error.stack;
  typed.cause = error;
  return typed;
}

/** Run `fn`, converting any engine failure into a typed error. */
export async function rethrow<T>(fn: () => Promise<T>): Promise<T> {
  try {
    return await fn();
  } catch (error) {
    throw toSoneError(error);
  }
}

/** The synchronous counterpart of `rethrow`. */
export function rethrowSync<T>(fn: () => T): T {
  try {
    return fn();
  } catch (error) {
    throw toSoneError(error);
  }
}
