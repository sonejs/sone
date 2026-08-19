using static Sone.Dsl;

namespace Sone.Sample;

/// <summary>
/// The single-page card from the README, as a PDF rather than a PNG — the tree
/// does not change, only the method called at the end.
/// </summary>
internal static class Card
{
    internal static Rendering Build(Engine engine) => Column(
            Column(
                Text("sone").Font("Google Sans").Size(34).Weight(FontWeight.Bold).Color("#14171a"),
                Text("layout · text · pixels")
                    .Font("Geist Mono").Size(11).Color("#66707c").LetterSpacing(1.5)
            ).Gap(6).Flex(1).Padding(24).JustifyContent(JustifyContent.Center)
                .CornerRadius(20).CornerSmoothing(0.7).Bg("white"),
            Row(
                Column().Bg("lightgreen").Size(50).BorderRadius(14),
                Column().Bg("salmon").Height(50).BorderRadius(14).Flex(1),
                Column().Bg("#7f8fa6").Size(50).BorderRadius(14)
            ).Gap(10)
        )
        .Gap(20)
        .Padding(20)
        .Size(420, 300)
        .Bg("khaki")
        .Render(engine);
}
