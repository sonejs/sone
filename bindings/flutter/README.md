# sone_flutter

Flutter packaging for [sone](../dart) — a declarative canvas layout engine with
rich international text.

The API is `package:sone`. This package exists to do the two things a Flutter
app needs that a command-line program does not: carry the native library into
the APK, and keep rendering off the UI isolate.

```dart
import 'package:sone_flutter/sone_flutter.dart' as s;

final font = await s.loadFontAsset('Inter', 'assets/Inter-Regular.ttf');

final root = s.Column(
  gap: 20, padding: 20, width: 420, height: 300,
  bg: 'khaki', cornerRadius: 28,
  children: [
    s.Text('Hello', font: 'Inter', size: 28),
    s.Row(gap: 10, children: [
      s.Column(bg: 'salmon', size: 50, cornerRadius: 14),
    ]),
  ],
);

final png = await s.render(root, density: 2).pngAsync(fonts: [font]);
// ... Image.memory(png)
```

## The two Flutter-shaped problems

**Fonts come from the asset bundle.** Skia carries no system fonts — not on
Android either — so a family has to be registered before any text renders.
`loadFontAsset` reads one through `rootBundle`.

**Every FFI call blocks the isolate it runs on**, and a render is long enough to
drop frames. The `…Async` methods run on a background isolate with an engine of
their own:

```dart
await s.render(root).pngAsync(fonts: [font]);
await s.render(root).pdfAsync(fonts: [font]);
await s.render(root, pageHeight: 1056).pagesAsync(fonts: [font]);
```

Only sendable values cross the boundary: the document as JSON, and fonts as
bytes. An `Engine` cannot be sent — it owns a native pointer — which is why the
fonts are passed in rather than registered once.

The synchronous API from `package:sone` is still there and still correct; it
just belongs off the UI isolate, or in a `flutter test`.

## Platforms

Android **arm64-v8a** and **x86_64**, minSdk 21.

`armeabi-v7a` is not built: rust-skia publishes no 32-bit Android binary, and
Play Store has required 64-bit since 2019. iOS is not wired up here — the Swift
package ships an XCFramework for iPhone and iPad, and pointing this package at
it is the remaining work.

## Building

The native libraries are not committed — they are ~19 MB each. Build them first:

```bash
tools/build-android.sh bindings/flutter/android/src/main/jniLibs
```

Then the example app runs normally:

```bash
cd bindings/flutter/example
flutter run
```

The Gradle module fails with a readable message rather than shipping an APK that
crashes on first render if the `.so` files are missing.

There is no CMake build here on purpose. Skia takes about an hour to compile
from source, so the libraries are cross-compiled ahead of time and this module
only hands them to the Android Gradle Plugin. `tools/build-android.sh` takes
about a minute because its feature set is chosen to match a prebuilt Skia that
rust-skia publishes — read the comment at the top of that script before changing
any feature.
