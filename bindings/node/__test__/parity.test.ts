/**
 * The binding renders exactly what `sone-cli` renders.
 *
 * Every sone binding is required to carry this test (`docs/bindings.md`): it is
 * cheap, and byte equality against the CLI rules out the whole class of
 * marshalling bugs — a dropped option, a mangled string, a density applied
 * twice.
 *
 * The CLI is built on demand, so the first run is slow. Set `SONE_SKIP_CLI=1`
 * to skip where cargo is not available.
 */
import { execFileSync } from "node:child_process";
import { existsSync } from "node:fs";
import { mkdtemp, readFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { beforeAll, describe, expect, it } from "vitest";

import { Engine } from "../ts/index.ts";
import { FIXTURES, REPO } from "./helpers.ts";

const skip = process.env.SONE_SKIP_CLI === "1";

/** Fixtures that need no fonts registered outside the document itself. */
const CASES = ["corners-1", "gradients-1", "grid-1", "path-1", "shadows-1"].filter(
  (name) => existsSync(`${FIXTURES}/visual/ir/${name}.json`),
);

let cli = "";

function cargo(args: string[]): string {
  return execFileSync("cargo", args, { cwd: REPO, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] });
}

describe.skipIf(skip)("byte parity with sone-cli", () => {
  beforeAll(() => {
    cargo(["build", "--release", "-p", "sone-cli"]);
    cli = join(REPO, "target", "release", "sone");
    if (!existsSync(cli)) cli = join(REPO, "target", "release", "sone-cli");
  }, 900_000);

  it.each(CASES)("%s renders identically", async (name) => {
    const source = `${FIXTURES}/visual/ir/${name}.json`;
    const document = await readFile(source, "utf8");

    const dir = await mkdtemp(join(tmpdir(), "sone-parity-"));
    const target = join(dir, `${name}.png`);
    execFileSync(cli, ["render", source, "-o", target], { cwd: REPO });
    const expected = new Uint8Array(await readFile(target));

    // The CLI resolves relative asset paths against the document's directory,
    // so the engine has to be given the same base.
    const engine = new Engine(`${FIXTURES}/visual/ir`);
    const actual = await engine.render(document, "png");

    expect(actual.length).toBe(expected.length);
    expect(Buffer.from(actual).equals(Buffer.from(expected))).toBe(true);
  });
});
