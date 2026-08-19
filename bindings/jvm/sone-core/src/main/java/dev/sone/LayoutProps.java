package dev.sone;

/** Flexbox, sizing, spacing and the visual box properties. */
public interface LayoutProps<SELF extends LayoutProps<SELF>> extends NodeProps<SELF> {

    default SELF alignContent(AlignContent value) { return set("alignContent", value.value()); }
    default SELF alignItems(AlignItems value) { return set("alignItems", value.value()); }
    default SELF alignSelf(AlignItems value) { return set("alignSelf", value.value()); }
    default SELF aspectRatio(double value) { return set("aspectRatio", value); }
    default SELF boxSizing(BoxSizing value) { return set("boxSizing", value.value()); }
    default SELF direction(Direction value) { return set("direction", value.value()); }
    default SELF display(Display value) { return set("display", value.value()); }
    default SELF flex(double value) { return set("flex", value); }
    default SELF basis(double value) { return set("flexBasis", value); }
    default SELF basis(Dim value) { return set("flexBasis", value); }
    default SELF flexDirection(FlexDirection value) { return set("flexDirection", value.value()); }
    default SELF grow(double value) { return set("flexGrow", value); }
    default SELF shrink(double value) { return set("flexShrink", value); }
    default SELF wrap(FlexWrap value) { return set("flexWrap", value.value()); }
    default SELF justifyContent(JustifyContent value) { return set("justifyContent", value.value()); }
    default SELF overflow(Overflow value) { return set("overflow", value.value()); }
    default SELF position(Position value) { return set("position", value.value()); }

    default SELF gap(double value) { return set("gap", value); }
    default SELF rowGap(double value) { return set("rowGap", value); }
    default SELF columnGap(double value) { return set("columnGap", value); }

    /** Width and height. One argument makes a square. */
    default SELF size(double value) { return size(value, value); }

    default SELF size(double width, double height) {
        return set("width", width).set("height", height);
    }

    default SELF size(Dim width, Dim height) {
        return set("width", width).set("height", height);
    }

    default SELF width(double value) { return set("width", value); }
    default SELF width(Dim value) { return set("width", value); }
    default SELF height(double value) { return set("height", value); }
    default SELF height(Dim value) { return set("height", value); }
    default SELF minWidth(double value) { return set("minWidth", value); }
    default SELF minWidth(Dim value) { return set("minWidth", value); }
    default SELF minHeight(double value) { return set("minHeight", value); }
    default SELF minHeight(Dim value) { return set("minHeight", value); }
    default SELF maxWidth(double value) { return set("maxWidth", value); }
    default SELF maxWidth(Dim value) { return set("maxWidth", value); }
    default SELF maxHeight(double value) { return set("maxHeight", value); }
    default SELF maxHeight(Dim value) { return set("maxHeight", value); }

    /**
     * CSS shorthand: one value for all four sides, or all four. Java has no
     * named arguments, so the two- and three-value CSS forms are spelled out
     * rather than inferred.
     */
    default SELF padding(double all) { return set("padding", all); }

    default SELF padding(double top, double right, double bottom, double left) {
        return set("paddingTop", top).set("paddingRight", right)
                .set("paddingBottom", bottom).set("paddingLeft", left);
    }

    default SELF padding(double vertical, double horizontal) {
        return padding(vertical, horizontal, vertical, horizontal);
    }

    default SELF margin(double all) { return set("margin", all); }

    default SELF margin(double top, double right, double bottom, double left) {
        return set("marginTop", top).set("marginRight", right)
                .set("marginBottom", bottom).set("marginLeft", left);
    }

    default SELF margin(double vertical, double horizontal) {
        return margin(vertical, horizontal, vertical, horizontal);
    }

    default SELF borderWidth(double all) { return set("borderWidth", all); }

    default SELF borderWidth(double top, double right, double bottom, double left) {
        return set("borderTopWidth", top).set("borderRightWidth", right)
                .set("borderBottomWidth", bottom).set("borderLeftWidth", left);
    }

    default SELF borderColor(String value) { return set("borderColor", value); }

    default SELF marginTop(double value) { return set("marginTop", value); }
    default SELF marginRight(double value) { return set("marginRight", value); }
    default SELF marginBottom(double value) { return set("marginBottom", value); }
    default SELF marginLeft(double value) { return set("marginLeft", value); }
    default SELF paddingTop(double value) { return set("paddingTop", value); }
    default SELF paddingRight(double value) { return set("paddingRight", value); }
    default SELF paddingBottom(double value) { return set("paddingBottom", value); }
    default SELF paddingLeft(double value) { return set("paddingLeft", value); }

    default SELF top(double value) { return set("top", value); }
    default SELF right(double value) { return set("right", value); }
    default SELF bottom(double value) { return set("bottom", value); }
    default SELF left(double value) { return set("left", value); }

    /** The leading inset, which flips with the writing direction. */
    default SELF start(double value) { return set("start", value); }

    /** The trailing inset, which flips with the writing direction. */
    default SELF end(double value) { return set("end", value); }

    default SELF inset(double value) { return set("inset", value); }

    default SELF gridColumn(int start) { return set("gridColumnStart", start); }

    default SELF gridColumn(int start, int span) {
        return set("gridColumnStart", start).set("gridColumnSpan", span);
    }

    default SELF gridRow(int start) { return set("gridRowStart", start); }

    default SELF gridRow(int start, int span) {
        return set("gridRowStart", start).set("gridRowSpan", span);
    }

    /** Force or forbid a page break at this node. Needs {@code pageHeight}. */
    default SELF pageBreak(PageBreakMode value) { return set("pageBreak", value.value()); }

    default SELF translateX(double value) { return set("translateX", value); }
    default SELF translateY(double value) { return set("translateY", value); }

    /** Rotation in degrees, about the node's centre. */
    default SELF rotate(double degrees) { return set("rotation", degrees); }

    /** Scale. One argument scales both axes. */
    default SELF scale(double value) { return scale(value, value); }

    default SELF scale(double x, double y) {
        return set("scale", java.util.List.of(x, y));
    }

    /** Add background layers: CSS colours, gradients, or a {@link Photo}. */
    default SELF bg(String... layers) { return push("background", (Object[]) layers); }

    default SELF bg(Photo photo) { return push("background", photo); }

    default SELF background(String... layers) { return bg(layers); }

    default SELF opacity(double value) { return set("opacity", value); }

    /** Corner radii: one value for all four, or up to four from the top left. */
    default SELF cornerRadius(double... radii) {
        java.util.List<Double> values = new java.util.ArrayList<>(radii.length);
        for (double radius : radii) {
            values.add(radius);
        }
        return set("cornerRadius", values);
    }

    default SELF rounded(double... radii) { return cornerRadius(radii); }

    default SELF borderRadius(double... radii) { return cornerRadius(radii); }

    /** Squircle-ness, 0..1. Figma's corner smoothing. */
    default SELF cornerSmoothing(double value) { return set("cornerSmoothing", value); }

    default SELF borderSmoothing(double value) { return set("cornerSmoothing", value); }

    default SELF corner(Corner value) { return set("corner", value.value()); }

    /** Add CSS {@code box-shadow} strings. */
    default SELF shadow(String... shadows) { return push("shadows", (Object[]) shadows); }

    // CSS filters, applied in the order they are added.
    default SELF blur(double radius) { return filter("blur(" + Num.of(radius) + "px)"); }
    default SELF brightness(double amount) { return filter("brightness(" + Num.of(amount) + ")"); }
    default SELF contrast(double amount) { return filter("contrast(" + Num.of(amount) + ")"); }
    default SELF grayscale(double amount) { return filter("grayscale(" + Num.of(amount) + ")"); }
    default SELF hueRotate(double degrees) { return filter("hue-rotate(" + Num.of(degrees) + ")"); }
    default SELF invert(double amount) { return filter("invert(" + Num.of(amount) + ")"); }
    default SELF saturate(double amount) { return filter("saturate(" + Num.of(amount) + ")"); }
    default SELF sepia(double amount) { return filter("sepia(" + Num.of(amount) + ")"); }

    default SELF filter(String css) { return push("filters", css); }
}
