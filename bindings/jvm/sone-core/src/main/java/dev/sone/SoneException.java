package dev.sone;

/** The base for every sone failure. */
public class SoneException extends RuntimeException {
    public SoneException(String message) {
        super(message);
    }
}
