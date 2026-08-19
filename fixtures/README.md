# fixtures

The parity corpus: everything the Rust engine is measured against. All of it is
generated from the TypeScript engine, which is the behavioural reference.

```
font/                 the eight faces the visual fixtures draw with
image/                images and SVGs the fixtures embed
visual/
  ir/*.json           IR documents, one per visual fixture
  layout/*.json       computed layout trees from the TypeScript engine
  break-corpus.json   every fixture string with its Intl.Segmenter breaks
  *.jpg *.pdf         golden renders from the TypeScript engine
```

`visual/ir/*.json` references assets with paths like `../../image/kouprey.jpg`,
which is why the directory depth here mirrors the TypeScript repo's `test/`.

Refresh with `tools/sync-fixtures.sh <path-to-sone-checkout>`.
