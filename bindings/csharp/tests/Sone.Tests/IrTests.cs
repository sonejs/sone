using System.Text.Json;
using Xunit;
using static Sone.Dsl;

namespace Sone.Tests;

/// <summary>
/// The builder layer, which touches no native code at all — the whole point of
/// the IR split is that this is testable without a rasterizer.
/// </summary>
public class IrTests
{
    private static JsonElement Document(INode root) =>
        JsonDocument.Parse(root.Render().Json()).RootElement;

    private static JsonElement Props(INode root) => Document(root).GetProperty("root").GetProperty("props");

    [Fact]
    public void EmitsTheSchemaVersionAndRoot()
    {
        var document = Document(Column());
        Assert.Equal(1, document.GetProperty("sone").GetInt32());
        Assert.Equal("column", document.GetProperty("root").GetProperty("type").GetString());
        Assert.False(document.TryGetProperty("config", out _));
    }

    [Fact]
    public void ChainingKeepsTheConcreteType()
    {
        // If T stopped inferring, this would only compile as INode.
        ColumnNode node = Column().Gap(10).Padding(20).Bg("khaki").Rounded(8);
        Assert.Equal(20.0, Props(node).GetProperty("padding").GetDouble());
    }

    [Fact]
    public void DimAcceptsNumbersPercentagesAndAuto()
    {
        var props = Props(Column().Width(100).MinWidth("50%").MaxWidth(Dim.Auto));
        Assert.Equal(100.0, props.GetProperty("width").GetDouble());
        Assert.Equal("50%", props.GetProperty("minWidth").GetString());
        Assert.Equal("auto", props.GetProperty("maxWidth").GetString());
    }

    [Fact]
    public void SizeWithOneArgumentIsASquare()
    {
        var props = Props(Column().Size(50));
        Assert.Equal(50.0, props.GetProperty("width").GetDouble());
        Assert.Equal(50.0, props.GetProperty("height").GetDouble());
    }

    [Fact]
    public void BoxShorthandFollowsCss()
    {
        var props = Props(Column().Padding(10, 20));
        Assert.Equal(10.0, props.GetProperty("paddingTop").GetDouble());
        Assert.Equal(20.0, props.GetProperty("paddingRight").GetDouble());
        Assert.Equal(10.0, props.GetProperty("paddingBottom").GetDouble());
        Assert.Equal(20.0, props.GetProperty("paddingLeft").GetDouble());
        Assert.False(props.TryGetProperty("padding", out _));
    }

    [Fact]
    public void NamedArgumentsFillTheMissingSidesTheCssWay()
    {
        var props = Props(Column().Padding(top: 8, left: 4));
        Assert.Equal(8.0, props.GetProperty("paddingTop").GetDouble());
        Assert.Equal(8.0, props.GetProperty("paddingRight").GetDouble());
        Assert.Equal(8.0, props.GetProperty("paddingBottom").GetDouble());
        Assert.Equal(4.0, props.GetProperty("paddingLeft").GetDouble());
    }

    [Fact]
    public void OneValueUsesTheShorthandProperty()
    {
        var props = Props(Column().Margin(12));
        Assert.Equal(12.0, props.GetProperty("margin").GetDouble());
        Assert.False(props.TryGetProperty("marginTop", out _));
    }

    [Fact]
    public void KeywordsAcceptStructsAndStrings()
    {
        var props = Props(Row().JustifyContent(JustifyContent.SpaceBetween).AlignItems("center"));
        Assert.Equal("space-between", props.GetProperty("justifyContent").GetString());
        Assert.Equal("center", props.GetProperty("alignItems").GetString());
    }

    [Fact]
    public void BackgroundLayersAccumulate()
    {
        var props = Props(Column().Bg("red").Bg("linear-gradient(red, blue)"));
        var layers = props.GetProperty("background");
        Assert.Equal(2, layers.GetArrayLength());
        Assert.Equal("red", layers[0].GetString());
    }

    [Fact]
    public void APhotoCanBeABackgroundLayer()
    {
        var layers = Props(Column().Bg(Photo("wall.png"))).GetProperty("background");
        Assert.Equal("photo", layers[0].GetProperty("type").GetString());
        Assert.Equal("wall.png", layers[0].GetProperty("props").GetProperty("src").GetString());
    }

    [Fact]
    public void FiltersKeepTheOrderTheyWereAddedIn()
    {
        var filters = Props(Column().Blur(4).Grayscale(0.5)).GetProperty("filters");
        Assert.Equal("blur(4px)", filters[0].GetString());
        Assert.Equal("grayscale(0.5)", filters[1].GetString());
    }

    [Fact]
    public void TextSizeIsTheFontSizeNotTheBoxSize()
    {
        // TextNode declares Size as an instance method precisely so this is the
        // span property and not the layout one.
        var props = Props(Text("Hello").Size(28));
        Assert.Equal(28.0, props.GetProperty("size").GetDouble());
        Assert.False(props.TryGetProperty("width", out _));
    }

    [Fact]
    public void TextWrapIsTheParagraphPropertyNotFlexWrap()
    {
        Assert.True(Props(Text("Hi").Wrap(false)).GetProperty("nowrap").GetBoolean());
        Assert.Equal("wrap", Props(Row().Wrap(FlexWrap.Wrap)).GetProperty("flexWrap").GetString());
    }

    [Fact]
    public void TextTakesStringsAndSpans()
    {
        var inline = Document(Text("Hello ", Span("world").Weight(FontWeight.Bold)))
            .GetProperty("root").GetProperty("inline");
        Assert.Equal("Hello ", inline[0].GetString());
        Assert.Equal("span", inline[1].GetProperty("type").GetString());
        Assert.Equal("bold", inline[1].GetProperty("props").GetProperty("weight").GetString());
    }

    [Fact]
    public void WeightTakesAKeywordOrANumber()
    {
        Assert.Equal(700.0, Props(Text("x").Weight(700)).GetProperty("weight").GetDouble());
    }

    [Fact]
    public void ADecorationColourCanBeExplicitlyNull()
    {
        // Null means "use the text colour", which is not the same as unset.
        var props = Props(Text("x").Underline().UnderlineColor());
        Assert.Equal(JsonValueKind.Null, props.GetProperty("underlineColor").ValueKind);
    }

    [Fact]
    public void NullChildrenAreDropped()
    {
        var showExtra = false;
        var children = Document(Column(Column(), showExtra ? Row() : null))
            .GetProperty("root").GetProperty("children");
        Assert.Equal(1, children.GetArrayLength());
    }

    [Fact]
    public void CollectionExpressionsSpreadIntoChildren()
    {
        string[] cells = ["a", "b", "c"];
        var row = TableRow([.. cells.Select(c => TableCell(Text(c)))]);
        Assert.Equal(3, Document(row).GetProperty("root").GetProperty("children").GetArrayLength());
    }

    [Fact]
    public void GridTracksAcceptFrAndAuto()
    {
        var columns = Props(Grid().Columns(Track.Fr(1), Track.Auto, 120)).GetProperty("columns");
        Assert.Equal("1fr", columns[0].GetString());
        Assert.Equal("auto", columns[1].GetString());
        Assert.Equal(120.0, columns[2].GetDouble());
    }

    [Fact]
    public void PhotoBytesBecomeADataUrl()
    {
        var src = Props(Photo(new byte[] { 1, 2, 3 })).GetProperty("src").GetString();
        Assert.StartsWith("data:application/octet-stream;base64,", src);
    }

    [Fact]
    public void PageBreakIsAZeroHeightColumn()
    {
        var props = Props(PageBreak());
        Assert.Equal(0.0, props.GetProperty("height").GetDouble());
        Assert.Equal("before", props.GetProperty("pageBreak").GetString());
    }

    [Fact]
    public void ConfigIsWrittenOnlyWhenSomethingIsSet()
    {
        var config = JsonDocument
            .Parse(Column().Render(width: 420, height: 300, margin: 20).Json())
            .RootElement.GetProperty("config");
        Assert.Equal(420.0, config.GetProperty("width").GetDouble());
        Assert.Equal(20.0, config.GetProperty("margin").GetProperty("top").GetDouble());
    }

    [Fact]
    public void HeadersKeepTheirPaginationTokens()
    {
        // The engine substitutes these; a binding must not.
        var json = Column().Render(pageHeight: 800, header: Text("Page {pageNumber}")).Json();
        Assert.Contains("{pageNumber}", json);
    }

    [Fact]
    public void NonAsciiTextSurvivesUnescaped()
    {
        Assert.Contains("អក្សរ", Text("អក្សរ").Render().Json());
    }

    [Fact]
    public void AnInvalidLengthIsRejectedAtTheCallSite()
    {
        Assert.Throws<ArgumentException>(() => Column().Width("wide"));
    }
}
