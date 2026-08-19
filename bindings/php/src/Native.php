<?php

declare(strict_types=1);

namespace Sone;

/**
 * The C ABI from `include/sone.h`. Nothing above this class sees a pointer.
 *
 * FFI rather than a compiled extension, so `composer require` needs no build
 * step. Two practical wrinkles are handled here: PHP's FFI parser is not a C
 * preprocessor, so it reads the stripped `sone.h` next to this file rather than
 * the canonical one; and the library has to be found at runtime, which is why
 * this uses `FFI::cdef()` rather than `FFI::load()`.
 */
final class Native
{
    public const OK = 0;
    public const INVALID_ARGUMENT = 1;
    public const IR_ERROR = 2;
    public const ASSET_ERROR = 3;
    public const RENDER_ERROR = 4;

    /** @var array<string,int> */
    public const FORMATS = [
        'png' => 0, 'jpeg' => 1, 'jpg' => 1, 'webp' => 2,
        'raw' => 3, 'rgba' => 3, 'pdf' => 4, 'svg' => 5,
    ];

    /** A full path to the library, or a directory holding it. */
    public const PATH_VARIABLE = 'SONE_NATIVE_LIBRARY';

    private static ?\FFI $ffi = null;

    public static function ffi(): \FFI
    {
        return self::$ffi ??= \FFI::cdef(
            file_get_contents(__DIR__ . '/../sone.h'),
            self::locate(),
        );
    }

    public static function fileName(): string
    {
        return match (PHP_OS_FAMILY) {
            'Windows' => 'sone.dll',
            'Darwin' => 'libsone.dylib',
            default => 'libsone.so',
        };
    }

    /**
     * An explicit hint first, then a `cargo build` in a checkout, then the
     * loader's own search path — which is what a released package uses.
     */
    public static function locate(): string
    {
        $name = self::fileName();
        $hint = getenv(self::PATH_VARIABLE);
        if (is_string($hint) && $hint !== '') {
            $candidate = is_dir($hint) ? "$hint/$name" : $hint;
            if (is_file($candidate)) {
                return $candidate;
            }
            throw new SoneException(self::PATH_VARIABLE . " is set to \"$hint\" but no $name is there");
        }

        $root = self::checkoutRoot();
        if ($root !== null) {
            foreach (['release', 'debug'] as $profile) {
                $candidate = "$root/target/$profile/$name";
                if (is_file($candidate)) {
                    return $candidate;
                }
            }
        }

        return $name;
    }

    /** The repository root, when this package is used from a checkout. */
    public static function checkoutRoot(): ?string
    {
        $directory = __DIR__;
        while (true) {
            if (is_file("$directory/Cargo.toml") && is_dir("$directory/crates")) {
                return $directory;
            }
            $parent = \dirname($directory);
            if ($parent === $directory) {
                return null;
            }
            $directory = $parent;
        }
    }
}
