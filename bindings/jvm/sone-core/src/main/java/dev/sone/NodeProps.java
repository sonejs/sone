package dev.sone;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Map;

/**
 * The self-type plumbing every property interface builds on.
 *
 * <p>{@code SELF} is what keeps {@code new Column().gap(20).padding(20)} typed
 * as a {@code Column} through the whole chain.
 */
public interface NodeProps<SELF extends NodeProps<SELF>> {

    /** Properties set on this node. */
    Map<String, Object> props();

    @SuppressWarnings("unchecked")
    default SELF self() {
        return (SELF) this;
    }

    /** A name for this node, echoed back by {@code layoutJson()}. */
    default SELF tag(String value) {
        return set("tag", value);
    }

    /** Set raw IR properties, for anything this API does not cover yet. */
    default SELF apply(Map<String, Object> values) {
        props().putAll(values);
        return self();
    }

    /** Set a property, ignoring nulls the way an omitted argument should be. */
    default SELF set(String key, Object value) {
        if (value != null) {
            props().put(key, value);
        }
        return self();
    }

    /**
     * Set a property that may legitimately be null — an explicit null clears a
     * decoration colour, which the engine reads differently from unset.
     */
    default SELF setNullable(String key, Object value) {
        props().put(key, value);
        return self();
    }

    /** Append to a list-valued property such as {@code background}. */
    @SuppressWarnings("unchecked")
    default SELF push(String key, Object... values) {
        List<Object> list = (List<Object>) props().computeIfAbsent(key, ignored -> new ArrayList<>());
        Collections.addAll(list, values);
        return self();
    }
}
