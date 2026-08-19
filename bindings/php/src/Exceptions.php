<?php

declare(strict_types=1);

namespace Sone;

/** The base for every sone failure. */
class SoneException extends \RuntimeException {}

/** The IR document could not be parsed. */
final class IrException extends SoneException {}

/** A font or an image could not be loaded. */
final class AssetException extends SoneException {}

/** Layout or rasterization failed. */
final class RenderException extends SoneException {}
