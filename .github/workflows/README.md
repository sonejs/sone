# Workflows

One per binding, each running that binding's own suite — which always includes
its parity gate: the same document rendered through the binding and through
`sone-cli` must come out byte for byte identical.

| workflow | Linux | macOS | Windows | notes |
|---|:-:|:-:|:-:|---|
| `rust` | ✓ | ✓ | ✓ | the workspace, including the `sone` builder crate |
| `csharp` | ✓ | ✓ | ✓ | |
| `ruby` | ✓ | ✓ | ✓ | plus a 2.7 job near the gemspec floor |
| `php` | ✓ | ✓ | ✓ | plus an 8.1 job at the composer floor |
| `dart` | ✓ | ✓ | ✓ | |
| `jvm` | ✓ | ✓ | ✓ | runs both backends: Panama and JNA |
| `node` | ✓ | ✓ | ✓ | |
| `python` | ✓ | ✓ | ✓ | |
| `swift` | — | ✓ | — | links an XCFramework of Apple slices |
| `android` | ✓ | — | — | cross-compiles, builds an APK, dexes the JVM artifacts |

## Two things every binding workflow does the same way

**Builds the native library in debug.** Each binding's parity gate spawns
`cargo run -p sone-cli`, which is a debug build, and the bindings prefer
`target/release` over `target/debug` when both exist. Building release here
would quietly have the binding and the CLI come from different profiles.

**Installs `libfontconfig1-dev` on Linux.** `embed-freetype` is on by default,
but Skia still links fontconfig for font enumeration there.

## Why the mobile feature sets differ

`tools/build-android.sh` and `tools/build-apple.sh` do **not** use the default
features, and that is deliberate. skia-bindings picks a prebuilt tarball by
hashing target + features, so the features have to name an asset rust-skia
actually publishes. Get it wrong and the build falls back to compiling Skia from
source — an hour per ABI when it works at all. Each script explains its own
choice in a header comment; read that before changing a feature.

Desktop builds keep `embed-freetype`, which no prebuilt carries, so the first
run on a fresh runner compiles Skia from source. `Swatinem/rust-cache` is what
makes the second run fast — and it is why the goldens are comparable across
runners at all, since embedding FreeType is what makes glyph rasterization
host-independent.
