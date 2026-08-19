// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "Sone",
    // Every Apple platform the XCFramework carries a slice for. iOS covers
    // iPadOS and Mac Catalyst; the simulator slice is in the same bundle.
    platforms: [
        .macOS(.v13),
        .iOS(.v13),
    ],
    products: [
        .library(name: "Sone", targets: ["Sone"])
    ],
    targets: [
        // Built by tools/build-apple.sh. A static XCFramework rather than a
        // dylib because iOS will not load an arbitrary one from a package, and
        // static linking is what lets the same Swift code serve every platform.
        .binaryTarget(name: "CSone", path: "Sone.xcframework"),
        .target(
            name: "Sone",
            dependencies: ["CSone"],
            linkerSettings: [
                // Skia's text and image stack reaches into the system
                // frameworks, and the exact set differs by platform. These are
                // the ones a link of the static slice actually demands.
                .linkedFramework("CoreFoundation"),
                .linkedFramework("CoreGraphics"),
                .linkedFramework("CoreText"),
                .linkedFramework("ImageIO", .when(platforms: [.iOS])),
                .linkedFramework("UIKit", .when(platforms: [.iOS])),
                .linkedFramework("Foundation", .when(platforms: [.iOS])),
                .linkedLibrary("c++"),
            ]
        ),
        .testTarget(name: "SoneTests", dependencies: ["Sone"]),
    ]
)
