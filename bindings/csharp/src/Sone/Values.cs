using System.Globalization;

namespace Sone;

/// <summary>
/// A length: a number, <c>auto</c>, or a percentage. Implicit conversions mean
/// <c>Width(100)</c>, <c>Width("50%")</c> and <c>Width(Dim.Auto)</c> are all the
/// same method.
/// </summary>
public readonly struct Dim : IEquatable<Dim>
{
    private readonly double _px;
    private readonly string? _text;

    private Dim(double px)
    {
        _px = px;
        _text = null;
    }

    private Dim(string text)
    {
        _px = 0;
        _text = text;
    }

    /// <summary>Size from content, the CSS <c>auto</c> keyword.</summary>
    public static readonly Dim Auto = new("auto");

    /// <summary>A percentage of the containing block.</summary>
    public static Dim Percent(double value) =>
        new(value.ToString("R", CultureInfo.InvariantCulture) + "%");

    public static implicit operator Dim(double value) => new(value);

    public static implicit operator Dim(string value) => Parse(value);

    private static Dim Parse(string value)
    {
        var text = value.Trim();
        if (text == "auto")
        {
            return Auto;
        }
        if (text.EndsWith('%'))
        {
            if (!double.TryParse(text[..^1], NumberStyles.Float, CultureInfo.InvariantCulture, out _))
            {
                throw new ArgumentException($"invalid percentage \"{value}\"", nameof(value));
            }
            return new Dim(text);
        }
        if (double.TryParse(text, NumberStyles.Float, CultureInfo.InvariantCulture, out var number))
        {
            return new Dim(number);
        }
        throw new ArgumentException($"invalid length \"{value}\" — expected a number, \"auto\" or a percentage", nameof(value));
    }

    internal object ToIr() => _text ?? (object)_px;

    public bool Equals(Dim other) => _text == other._text && _px.Equals(other._px);

    public override bool Equals(object? obj) => obj is Dim other && Equals(other);

    public override int GetHashCode() => HashCode.Combine(_px, _text);

    public override string ToString() => _text ?? _px.ToString("R", CultureInfo.InvariantCulture);

    public static bool operator ==(Dim left, Dim right) => left.Equals(right);

    public static bool operator !=(Dim left, Dim right) => !left.Equals(right);
}

/// <summary>
/// A grid track: a fixed size, <c>auto</c>, or an <c>fr</c> share.
/// </summary>
public readonly struct Track : IEquatable<Track>
{
    private readonly double _size;
    private readonly string? _text;

    private Track(double size)
    {
        _size = size;
        _text = null;
    }

    private Track(string text)
    {
        _size = 0;
        _text = text;
    }

    /// <summary>A track sized to its content.</summary>
    public static readonly Track Auto = new("auto");

    /// <summary>A share of the free space, the CSS <c>fr</c> unit.</summary>
    public static Track Fr(double value) =>
        new(value.ToString("R", CultureInfo.InvariantCulture) + "fr");

    public static implicit operator Track(double value) => new(value);

    public static implicit operator Track(string value) => Parse(value);

    private static Track Parse(string value)
    {
        var text = value.Trim();
        if (text == "auto")
        {
            return Auto;
        }
        if (text.EndsWith("fr", StringComparison.Ordinal)
            && double.TryParse(text[..^2], NumberStyles.Float, CultureInfo.InvariantCulture, out _))
        {
            return new Track(text);
        }
        if (double.TryParse(text, NumberStyles.Float, CultureInfo.InvariantCulture, out var number))
        {
            return new Track(number);
        }
        throw new ArgumentException($"invalid grid track \"{value}\" — expected a number, \"auto\" or an fr value", nameof(value));
    }

    internal object ToIr() => _text ?? (object)_size;

    public bool Equals(Track other) => _text == other._text && _size.Equals(other._size);

    public override bool Equals(object? obj) => obj is Track other && Equals(other);

    public override int GetHashCode() => HashCode.Combine(_size, _text);

    public override string ToString() => _text ?? _size.ToString("R", CultureInfo.InvariantCulture);

    public static bool operator ==(Track left, Track right) => left.Equals(right);

    public static bool operator !=(Track left, Track right) => !left.Equals(right);
}

/// <summary>
/// A font weight: a CSS keyword or a number. <c>Weight("bold")</c> and
/// <c>Weight(700)</c> both work.
/// </summary>
public readonly struct FontWeight : IEquatable<FontWeight>
{
    private readonly double _number;
    private readonly string? _keyword;

    private FontWeight(double number)
    {
        _number = number;
        _keyword = null;
    }

    private FontWeight(string keyword)
    {
        _number = 0;
        _keyword = keyword;
    }

    public static readonly FontWeight Normal = new("normal");
    public static readonly FontWeight Bold = new("bold");
    public static readonly FontWeight Lighter = new("lighter");
    public static readonly FontWeight Bolder = new("bolder");

    public static implicit operator FontWeight(double value) => new(value);

    public static implicit operator FontWeight(string value) => new(value);

    internal object ToIr() => _keyword ?? (object)_number;

    public bool Equals(FontWeight other) => _keyword == other._keyword && _number.Equals(other._number);

    public override bool Equals(object? obj) => obj is FontWeight other && Equals(other);

    public override int GetHashCode() => HashCode.Combine(_number, _keyword);

    public override string ToString() => _keyword ?? _number.ToString("R", CultureInfo.InvariantCulture);

    public static bool operator ==(FontWeight left, FontWeight right) => left.Equals(right);

    public static bool operator !=(FontWeight left, FontWeight right) => !left.Equals(right);
}

/// <summary>
/// A background layer: a CSS colour or gradient string, or a <see cref="PhotoNode"/>.
/// </summary>
public readonly struct Paint
{
    private readonly string? _css;
    private readonly PhotoNode? _photo;

    private Paint(string css)
    {
        _css = css;
        _photo = null;
    }

    private Paint(PhotoNode photo)
    {
        _css = null;
        _photo = photo;
    }

    public static implicit operator Paint(string css) => new(css);

    public static implicit operator Paint(PhotoNode photo) => new(photo);

    internal object ToIr() => (object?)_css ?? _photo!;

    public override string ToString() => _css ?? "<photo>";
}

/// <summary>
/// A piece of a paragraph: a raw string or a styled <see cref="SpanNode"/>.
/// </summary>
public readonly struct Inline
{
    private readonly string? _text;
    private readonly SpanNode? _span;

    private Inline(string text)
    {
        _text = text;
        _span = null;
    }

    private Inline(SpanNode span)
    {
        _text = null;
        _span = span;
    }

    public static implicit operator Inline(string text) => new(text);

    public static implicit operator Inline(SpanNode span) => new(span);

    internal object ToIr() => (object?)_text ?? _span!;

    public override string ToString() => _text ?? "<span>";
}

/// <summary>
/// Page margins. A single number applies to all four sides:
/// <c>margin: 20</c> and <c>margin: new Margin(top: 40, bottom: 40)</c> are both valid.
/// </summary>
public readonly record struct Margin(double Top = 0, double Right = 0, double Bottom = 0, double Left = 0)
{
    public static implicit operator Margin(double all) => new(all, all, all, all);

    internal Dictionary<string, object?> ToIr() => new()
    {
        ["top"] = Top,
        ["right"] = Right,
        ["bottom"] = Bottom,
        ["left"] = Left,
    };
}
