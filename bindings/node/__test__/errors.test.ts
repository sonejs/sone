/**
 * One error class per failure class, mapped from `SoneError::exit_code()`.
 *
 * The engine cannot set a JavaScript error `code` (Node-API fills it from the
 * napi status), so the class travels as a `sone:<class>:` message prefix that
 * `ts/errors.ts` strips. These tests are what keep that contract honest.
 */
import { describe, expect, it } from "vitest";

import {
  AssetError,
  Column,
  Engine,
  IrError,
  Photo,
  RenderError,
  sone,
  SoneError,
} from "../ts/index.ts";

const engine = () => new Engine();

describe("error classes", () => {
  it("raises IrError for an unsupported document version", async () => {
    const error = await new Engine()
      .render('{"sone":99,"root":{"type":"column"}}', "png")
      .catch((e: unknown) => e);
    expect(error).toBeInstanceOf(IrError);
    expect((error as Error).message).toContain("unsupported IR version");
    expect((error as Error).message).not.toContain("sone:ir:");
  });

  it("raises AssetError for an image that is not there", async () => {
    const error = await sone(Photo("does/not/exist.png").size(10), {
      engine: engine(),
    })
      .png()
      .catch((e: unknown) => e);
    expect(error).toBeInstanceOf(AssetError);
  });

  it("raises AssetError for a font file that is not there", async () => {
    await expect(
      engine().registerFontFile("Nope", "does/not/exist.ttf"),
    ).rejects.toBeInstanceOf(AssetError);
  });

  it("raises RenderError for a format the engine does not know", async () => {
    await expect(
      new Engine().render('{"sone":1,"root":{"type":"column"}}', "bmp"),
    ).rejects.toBeInstanceOf(RenderError);
  });

  it("nests every class under SoneError", () => {
    for (const Cls of [IrError, AssetError, RenderError]) {
      expect(new Cls("x")).toBeInstanceOf(SoneError);
      expect(new Cls("x")).toBeInstanceOf(Error);
      expect(new Cls("x").name).toBe(Cls.name);
    }
  });

  it("keeps the original error as the cause", async () => {
    const error = (await new Engine()
      .render('{"sone":99,"root":{"type":"column"}}', "png")
      .catch((e: unknown) => e)) as Error;
    expect((error.cause as Error).message).toMatch(/^sone:ir:/);
  });
});

describe("what the engine will not do for you", () => {
  it("refuses a remote asset the caller did not fetch", async () => {
    // `resolveAssets` normally fetches these first; go around it to prove the
    // engine itself still refuses, which is what makes renders network-free.
    const error = await new Engine()
      .render(
        JSON.stringify({
          sone: 1,
          root: { type: "photo", props: { src: "https://example.com/a.png", width: 10 } },
        }),
        "png",
      )
      .catch((e: unknown) => e);
    expect(error).toBeInstanceOf(AssetError);
    expect((error as Error).message).toContain("registered as asset:");
  });

  it("names the offending path in strict mode", async () => {
    // Strict mode is `serde_path_to_error`: same acceptances, better messages.
    // An unknown *config* key is ignored either way; an unknown node type is
    // rejected either way, but only strict says where it was.
    const bad = JSON.stringify({ sone: 1, root: { type: "nope" } });
    const strict = (await new Engine()
      .render(bad, "png", undefined, 1, true)
      .catch((e: unknown) => e)) as Error;
    expect(strict).toBeInstanceOf(IrError);
    expect(strict.message).toContain("root.type");

    const withUnknownConfig = JSON.stringify({
      sone: 1,
      config: { nonsense: 1 },
      root: { type: "column", props: { width: 4, height: 4 } },
    });
    await expect(
      new Engine().render(withUnknownConfig, "png", undefined, 1, true),
    ).resolves.toBeInstanceOf(Uint8Array);
  });

  it("leaves a non-engine failure alone", async () => {
    await expect(sone(Column()).save("/tmp/out.bmp")).rejects.toBeInstanceOf(TypeError);
  });
});
