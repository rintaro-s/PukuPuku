# Character Images Setup

## 🎨 Image Placement

Place your three character images in the `resources/characters/` directory:

```
resources/
└── characters/
    ├── normal.png       - PC is doing fine
    ├── very_hard.png    - Working hard
    └── danger.png       - Might be in trouble
```

## 📐 Recommended Specifications

- **Format**: PNG with transparency
- **Size**: 72x72 pixels or larger (will be scaled)
- **Style**: Hand-drawn, casual illustration
- **Background**: Transparent

## 🎭 Character Mood Triggers

The character changes based on system load:

### Normal (normal.png)
- CPU < 60%
- Memory < 75%
- Disk < 75%

### Very Hard (very_hard.png)
- CPU 60-89% OR
- Memory 75-89% OR
- Disk 75-89%

### Danger (danger.png)
- CPU ≥ 90% OR
- Memory ≥ 90% OR
- Disk ≥ 90%

## 🔧 Integration Steps

1. **Create directory:**
   ```bash
   mkdir -p resources/characters
   ```

2. **Add your images:**
   ```bash
   cp /path/to/your/normal.png resources/characters/
   cp /path/to/your/very_hard.png resources/characters/
   cp /path/to/your/danger.png resources/characters/
   ```

3. **Register in gresource:**
   
   Edit `resources/pukupuku.gresource.xml` and add inside the main `<gresource>` tag:
   
   ```xml
   <file>characters/normal.png</file>
   <file>characters/very_hard.png</file>
   <file>characters/danger.png</file>
   ```

4. **Rebuild:**
   ```bash
   ninja -C build-meson-debug
   ```

## 🎨 Design Tips

### Style Guidelines
- Keep it simple and recognizable
- Use clear expressions (happy → focused → worried)
- Maintain consistent character design across all three
- Avoid too much detail (will be displayed small)

### Color Suggestions
- **Normal**: Bright, warm colors (yellows, soft greens)
- **Very Hard**: Active colors (oranges, vibrant blues)
- **Danger**: Alert colors (reds, dark purples)

### Expression Ideas
- **Normal**: Smiling, relaxed, waving
- **Very Hard**: Focused, sweating slightly, working
- **Danger**: Worried, alarmed, steam coming out

## 🖼️ Example Structure

```
resources/characters/
├── normal.png       ← 😊 Happy, relaxed expression
├── very_hard.png    ← 😓 Focused, working hard
└── danger.png       ← 😰 Worried, stressed
```

## ✅ Verification

After adding images, verify they load correctly:

```bash
# Check files exist
ls -lh resources/characters/

# Build and run
./BUILD_INSTRUCTIONS.md  # Follow build steps
```

The character should appear at the top of the window and change based on system load.

## 🔍 Troubleshooting

### Images not appearing
1. Check file names match exactly (case-sensitive)
2. Verify they're registered in `pukupuku.gresource.xml`
3. Rebuild the project
4. Check file permissions (should be readable)

### Character not changing
- The mood updates with system readings (every ~1 second)
- Try loading the system (run a benchmark or heavy task)

### Image looks distorted
- Use square or near-square aspect ratios
- PNG format recommended
- Include transparency for best results

## 💡 Optional Enhancements

If you want to customize further, you can edit:
- `src/widgets/character_status.rs` - Mood logic and thresholds
- `resources/ui/widgets/character_status.blp` - Widget layout and size
- `resources/ui/style.css` - Add custom styling for the character widget
