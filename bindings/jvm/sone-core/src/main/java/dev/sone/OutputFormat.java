package dev.sone;

/** The output formats the engine can encode. */
public enum OutputFormat {
    PNG("png", 0),
    JPEG("jpeg", 1),
    WEBP("webp", 2),
    RAW("raw", 3),
    PDF("pdf", 4),
    SVG("svg", 5);

    private final String value;
    private final int code;

    OutputFormat(String value, int code) {
        this.value = value;
        this.code = code;
    }

    public String value() {
        return value;
    }

    /** The `SoneFormat` discriminant the C ABI expects. */
    public int code() {
        return code;
    }
}
