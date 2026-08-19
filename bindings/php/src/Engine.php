<?php

declare(strict_types=1);

namespace Sone;

/**
 * Owns the font registry and the decoded-image cache.
 *
 * Skia's font collection is shared inside an engine, so one engine renders one
 * document at a time. PHP has no threads to guard against, but the rule still
 * holds for anything that forks a request into workers.
 */
final class Engine
{
    private \FFI $ffi;
    private \FFI\CData $handle;
    private bool $closed = false;

    private static ?self $default = null;

    /** @param string|null $baseDir the directory relative asset paths resolve against */
    public function __construct(?string $baseDir = null)
    {
        $this->ffi = Native::ffi();
        $handle = $this->ffi->sone_engine_new($baseDir ?? getcwd());
        if ($handle === null) {
            throw new SoneException('could not create a sone engine');
        }
        $this->handle = $handle;
    }

    public function __destruct()
    {
        $this->close();
    }

    /** The process-wide engine, used when no explicit one is passed. */
    public static function default(): self
    {
        return self::$default ??= new self();
    }

    public static function version(): string
    {
        return Native::ffi()->sone_version();
    }

    public function close(): void
    {
        if (!$this->closed) {
            $this->closed = true;
            $this->ffi->sone_engine_free($this->handle);
        }
    }

    // ── fonts and assets ────────────────────────────────────────────────────

    /** Register a font family from raw TTF/OTF bytes. */
    public function registerFont(string $name, string $data): void
    {
        $this->withBytes($data, fn (\FFI\CData $buffer, int $length) =>
            $this->check($this->ffi->sone_register_font($this->live(), $name, $buffer, $length)));
    }

    /** Register a font family from a file. */
    public function registerFontFile(string $name, string $path): void
    {
        $this->check($this->ffi->sone_register_font_file($this->live(), $name, $path));
    }

    /** Make bytes available to documents as `asset:name`. */
    public function registerImage(string $name, string $data): void
    {
        $this->withBytes($data, fn (\FFI\CData $buffer, int $length) =>
            $this->check($this->ffi->sone_register_image($this->live(), $name, $buffer, $length)));
    }

    public function hasFont(string $name): bool
    {
        return $this->ffi->sone_has_font($this->live(), $name);
    }

    /** @return list<string> */
    public function fontFamilies(): array
    {
        return json_decode($this->buffer(fn (\FFI\CData $out) =>
            $this->ffi->sone_font_families($this->live(), $out)), true);
    }

    public function resetFonts(): void
    {
        $this->ffi->sone_reset_fonts($this->live());
    }

    // ── rendering ───────────────────────────────────────────────────────────

    /** Render an IR document to bytes. */
    public function render(
        string $document,
        OutputFormat|string $format = OutputFormat::Png,
        ?float $density = null,
        float $quality = 1.0,
        bool $strict = false,
    ): string {
        $options = $this->options($format, $density, $quality, $strict);

        return $this->buffer(fn (\FFI\CData $out) =>
            $this->ffi->sone_render_json($this->live(), $document, \FFI::addr($options), $out));
    }

    /**
     * One raster image per page. Requires `pageHeight` in the document config.
     *
     * @return list<string>
     */
    public function renderPages(
        string $document,
        OutputFormat|string $format = OutputFormat::Png,
        ?float $density = null,
        float $quality = 1.0,
        bool $strict = false,
    ): array {
        $options = $this->options($format, $density, $quality, $strict);
        $list = $this->ffi->new('SoneBufferList');
        try {
            $this->check($this->ffi->sone_render_pages(
                $this->live(),
                $document,
                \FFI::addr($options),
                \FFI::addr($list),
            ));
            $pages = [];
            for ($index = 0; $index < $list->len; $index++) {
                $page = $list->items[$index];
                $pages[] = $page->len > 0 ? \FFI::string($page->data, $page->len) : '';
            }

            return $pages;
        } finally {
            $this->ffi->sone_buffer_list_free(\FFI::addr($list));
        }
    }

    /** The computed layout tree, as JSON. */
    public function dumpLayout(string $document): string
    {
        return $this->buffer(fn (\FFI\CData $out) =>
            $this->ffi->sone_dump_layout($this->live(), $document, $out));
    }

    /** Dataset-style metadata, as JSON. */
    public function dumpMetadata(string $document, Granularity|string $granularity = Granularity::Node): string
    {
        $name = $granularity instanceof Granularity ? $granularity->value : $granularity;

        return $this->buffer(fn (\FFI\CData $out) =>
            $this->ffi->sone_dump_metadata($this->live(), $document, $name, $out));
    }

    // ── internals ───────────────────────────────────────────────────────────

    private function live(): \FFI\CData
    {
        if ($this->closed) {
            throw new SoneException('this engine has been closed');
        }

        return $this->handle;
    }

    /**
     * The options struct, always handed over as a pointer.
     *
     * It used to be passed by value, which segfaulted PHP on Linux x86-64 while
     * working on macOS and Windows — the C ABI takes a pointer now.
     */
    private function options(OutputFormat|string $format, ?float $density, float $quality, bool $strict): \FFI\CData
    {
        $name = strtolower($format instanceof OutputFormat ? $format->value : $format);
        $code = Native::FORMATS[$name] ?? throw new \InvalidArgumentException("unknown output format \"$name\"");

        $options = $this->ffi->new('SoneRenderOptions');
        $options->format = $code;
        // Zero tells the engine to fall back to the document's own config.
        $options->density = $density ?? 0.0;
        $options->quality = $quality;
        $options->strict = $strict ? 1 : 0;

        return $options;
    }

    private function withBytes(string $data, \Closure $body): void
    {
        $length = \strlen($data);
        // Instance calls, not FFI::new()/FFI::cast(): the static forms are
        // deprecated as of PHP 8.5.
        $buffer = $this->ffi->new('uint8_t[' . max($length, 1) . ']', false);
        try {
            if ($length > 0) {
                \FFI::memcpy($buffer, $data, $length);
            }
            $body($this->ffi->cast('uint8_t*', $buffer), $length);
        } finally {
            \FFI::free($buffer);
        }
    }

    private function buffer(\Closure $call): string
    {
        $out = $this->ffi->new('SoneBuffer');
        try {
            $this->check($call(\FFI::addr($out)));

            return $out->len > 0 ? \FFI::string($out->data, $out->len) : '';
        } finally {
            $this->ffi->sone_buffer_free(\FFI::addr($out));
        }
    }

    private function check(int $status): void
    {
        if ($status === Native::OK) {
            return;
        }
        $pointer = $this->ffi->sone_engine_last_error($this->handle);
        $message = $pointer === null ? "sone failed with status $status" : $pointer;

        throw match ($status) {
            Native::INVALID_ARGUMENT => new \InvalidArgumentException($message),
            Native::IR_ERROR => new IrException($message),
            Native::ASSET_ERROR => new AssetException($message),
            default => new RenderException($message),
        };
    }
}

/**
 * Font registration on the process-wide engine.
 *
 * Skia carries no system fonts, so at least one family must be registered
 * before any text renders.
 */
final class Font
{
    public static function load(string $name, string $path): void
    {
        Engine::default()->registerFontFile($name, $path);
    }

    public static function loadBytes(string $name, string $data): void
    {
        Engine::default()->registerFont($name, $data);
    }

    public static function has(string $name): bool
    {
        return Engine::default()->hasFont($name);
    }

    /** @return list<string> */
    public static function families(): array
    {
        return Engine::default()->fontFamilies();
    }

    public static function reset(): void
    {
        Engine::default()->resetFonts();
    }
}
