namespace Sone;

/// <summary>The base for every sone failure.</summary>
public class SoneException : Exception
{
    public SoneException(string message) : base(message) { }
}

/// <summary>The IR document could not be parsed.</summary>
public sealed class IrException : SoneException
{
    public IrException(string message) : base(message) { }
}

/// <summary>A font or an image could not be loaded.</summary>
public sealed class AssetException : SoneException
{
    public AssetException(string message) : base(message) { }
}

/// <summary>Layout or rasterization failed.</summary>
public sealed class RenderException : SoneException
{
    public RenderException(string message) : base(message) { }
}
