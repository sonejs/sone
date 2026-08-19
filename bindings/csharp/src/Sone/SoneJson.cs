using System.Text.Json.Serialization;

namespace Sone;

/// <summary>
/// Source-generated contracts for the little JSON this binding reads back from
/// the engine. Reflection-based serialization would work but would cost the
/// assembly its trimming and NativeAOT guarantees.
/// </summary>
[JsonSerializable(typeof(string[]))]
internal sealed partial class SoneJson : JsonSerializerContext
{
}
