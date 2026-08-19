package dev.sone;

import java.util.ArrayList;
import java.util.List;

/** An SVG path. Named {@code SvgPath} because {@code Path} is {@code java.nio.file.Path}. */
public final class SvgPath extends Node implements LayoutProps<SvgPath> {

    public SvgPath(String d) {
        super("path");
        set("d", d);
    }

    public SvgPath stroke(String color) { return set("stroke", color); }
    public SvgPath strokeWidth(double value) { return set("strokeWidth", value); }
    public SvgPath strokeLineCap(StrokeCap value) { return set("strokeLineCap", value.value()); }
    public SvgPath strokeLineJoin(StrokeJoin value) { return set("strokeLineJoin", value.value()); }
    public SvgPath strokeMiterLimit(double value) { return set("strokeMiterLimit", value); }
    public SvgPath strokeDashOffset(double value) { return set("strokeDashOffset", value); }
    public SvgPath fill(String color) { return set("fill", color); }
    public SvgPath fillOpacity(double value) { return set("fillOpacity", value); }
    public SvgPath fillRule(FillRule value) { return set("fillRule", value.value()); }

    /** Scale the path data itself, before layout. */
    public SvgPath scalePath(double value) { return set("scalePath", value); }

    public SvgPath strokeDashArray(double... values) {
        List<Double> list = new ArrayList<>(values.length);
        for (double value : values) {
            list.add(value);
        }
        return set("strokeDashArray", list);
    }
}
