# Character Images

Place your character images here:

- `normal.png` - PC is doing fine (displayed when CPU < 60%, Memory < 75%, Disk < 75%)
- `very_hard.png` - Working hard (displayed when any resource is 60-89% loaded)
- `danger.png` - Might be in trouble (displayed when any resource is ≥ 90% loaded)

## Image Specifications

- **Format**: PNG with transparency
- **Size**: 72x72 pixels or larger (will be scaled)
- **Style**: Hand-drawn, casual illustration

## After Adding Images

Uncomment the lines in `../pukupuku.gresource.xml`:

```xml
<file>characters/normal.png</file>
<file>characters/very_hard.png</file>
<file>characters/danger.png</file>
```

Then rebuild the project.

See CHARACTER_IMAGES.md in the project root for detailed instructions.
