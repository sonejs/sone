package dev.sone;

/** Whether width and height include padding and border. */
public enum BoxSizing {
    BORDER_BOX("border-box"),
    CONTENT_BOX("content-box");

    private final String value;

    BoxSizing(String value) {
        this.value = value;
    }

    public String value() {
        return value;
    }
}
