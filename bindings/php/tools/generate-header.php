<?php
/**
 * Rewrites include/sone.h into a header PHP's FFI parser can read.
 *
 * PHP's FFI parser is not a C preprocessor: it chokes on `#include` and on the
 * `extern "C"` guards. Everything else in the generated header is already
 * plain C, so this only has to strip.
 *
 *     php tools/generate-header.php
 */
declare(strict_types=1);

$root = dirname(__DIR__, 3);
$source = file_get_contents("$root/include/sone.h");
if ($source === false) {
    fwrite(STDERR, "could not read include/sone.h\n");
    exit(1);
}

$lines = [];
$skippingCpp = false;
foreach (explode("\n", $source) as $line) {
    $trimmed = trim($line);
    if (str_starts_with($trimmed, '#ifdef __cplusplus')) {
        $skippingCpp = true;
        continue;
    }
    if ($skippingCpp) {
        // The guarded body is a single `extern "C" {` or `}` line.
        if (str_starts_with($trimmed, '#endif')) {
            $skippingCpp = false;
        }
        continue;
    }
    if (str_starts_with($trimmed, '#')) {
        continue;
    }
    $lines[] = $line;
}

$body = preg_replace("/\n{3,}/", "\n\n", implode("\n", $lines));
// No FFI_SCOPE/FFI_LIB defines: the binding uses FFI::cdef() so it can find
// the library at runtime rather than pinning a path at build time.
$header = "/* Generated from include/sone.h by tools/generate-header.php — do not edit. */\n\n"
    . trim($body) . "\n";

file_put_contents(dirname(__DIR__) . '/sone.h', $header);
echo "wrote bindings/php/sone.h\n";
