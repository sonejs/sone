package dev.sone;

/** A font or an image could not be loaded. */
public final class AssetException extends SoneException {
    public AssetException(String message) {
        super(message);
    }
}
