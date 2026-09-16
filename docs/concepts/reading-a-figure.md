# Reading a figure

Every diagram Integrase draws uses two independent visual channels. Shape and
colour say what a part is. Fill texture and border style say how much the
prediction about it can be trusted. The channels never overlap, which is what
lets a figure survive greyscale printing and colour vision deficiency with its
uncertainty intact.

## Shape: what the part is

| Shape | Part |
|---|---|
| Bent arrow rising from the backbone | Promoter |
| Dome sitting on the backbone | Ribosome entry site |
| Arrow-shaped box | Coding sequence |
| T-bar | Terminator |
| Circle | Origin of replication |
| Plain box | A recognised feature with no distinctive glyph |
| Box with a diagonal stroke through it | **The role was not recognised** |

The last row is the one to read carefully. A struck-through box does not mean
"a part with nothing special about it". It means the file gave a role that this
version of Integrase has no glyph for, so the diagram is not making a claim
about what that region is. Hovering over it shows the raw ontology IRI.

Shapes follow SBOL Visual conventions. The struck-through box is ours, not part
of that standard.

Direction follows the strand. A reverse-strand coding sequence points left. It
still occupies the same span as a forward one at the same coordinates, because
the span is a fact about the sequence and the direction is a fact about the
feature.

## Texture: how much to trust it

| Fill | Border | Interval bar | Meaning |
|---|---|---|---|
| Solid | Solid | Solid caps | Inside the model's validated range |
| Diagonal hatching | Solid | Hatched caps | Near the edge of that range |
| Stipple | Dashed | Open caps | Outside it |
| None, outline only | Dotted | **No bar at all** | Not calibrated |

The bottom row is the default for a design read from a file, because a file
carries a sequence and not a prediction.

**No bar at all is a statement, not a missing feature.** A model with no
calibration evidence has no honest interval to draw. Drawing a wide one would
imply a quantified uncertainty that has not been earned, so the diagram draws
nothing and says so in the text alternative instead. See
[uncertainty](uncertainty.md) for what the intervals mean when they are present.

## Why texture and not colour

The four semantic colours in the palette measure 1.14:1 to 1.70:1 against each
other in contrast ratio, all far below the 3:1 threshold at which two colours
are reliably distinguishable by luminance. Hue alone cannot carry confidence for
anyone with a colour vision deficiency, and it cannot carry it at all in a
greyscale print. Since uncertainty is the thing this system exists to
communicate, it goes in the channel that always survives.

The test for the whole grammar is simple: convert a figure to greyscale. Shape
still says what the parts are. Texture still says how much to trust them.

## Text alternative

Every rendered figure carries a `<title>` naming the design and a `<desc>`
giving its length, its feature count, and its confidence state in words. Screen
readers get the confidence state, not only the geometry.

## Producing a figure

```
integrase render design.ttl --out figure.svg
```

When a file holds more than one design, the command refuses and lists the
identifiers rather than picking one. A figure of the wrong construct looks
exactly like a figure of the right one.

Rendering is not gated on biosecurity screening. Drawing a picture of a sequence
is not a step toward making one, and gating it would push people toward viewers
that carry no confidence encoding at all.
