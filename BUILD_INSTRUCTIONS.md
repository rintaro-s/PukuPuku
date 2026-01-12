# PukuPuku - Build Instructions

## 🎨 Character Images Setup (Important!)

Before building, place your character images in `resources/characters/`:

```bash
mkdir -p resources/characters
# Copy your images:
# - resources/characters/normal.png
# - resources/characters/very_hard.png
# - resources/characters/danger.png
```

Then register them in `resources/pukupuku.gresource.xml`:

```xml
<gresource prefix="/io/github/rinta/PukuPuku">
    <!-- ... existing files ... -->
    
    <file>characters/normal.png</file>
    <file>characters/very_hard.png</file>
    <file>characters/danger.png</file>
</gresource>
```

## 📦 Dependencies

### Ubuntu 25.10 / Debian-based

```bash
sudo apt install build-essential cmake curl desktop-file-utils gettext git \
    libadwaita-1-dev libdbus-1-dev libdrm-dev libgbm-dev libudev-dev \
    meson pkg-config protobuf-compiler python3-gi python3-pip
```

### Fedora 43+

```bash
sudo dnf install meson cmake rust cargo gcc gcc-c++ gettext desktop-file-utils \
    libadwaita-devel dbus-devel libdrm-devel mesa-libgbm-devel systemd-devel \
    protobuf-compiler python3-gobject
```

### Arch Linux

```bash
sudo pacman -S base-devel meson cmake rust cargo gettext desktop-file-utils \
    libadwaita dbus libdrm mesa systemd protobuf python-gobject
```

## 🔨 Building

### Debug Build (for development)

```bash
BUILD_ROOT="$(pwd)/build-meson-debug"

meson setup "$BUILD_ROOT" -Dbuildtype=debug
ninja -C "$BUILD_ROOT"
```

### Release Build (optimized)

```bash
BUILD_ROOT="$(pwd)/build-meson-release"

meson setup "$BUILD_ROOT" -Dbuildtype=release
ninja -C "$BUILD_ROOT"
```

## 🚀 Running (Development)

Set up the environment:

```bash
export PATH="$BUILD_ROOT/subprojects/magpie/src:$PATH"
export GSETTINGS_SCHEMA_DIR="$BUILD_ROOT/data"
export MC_MAGPIE_HW_DB="$BUILD_ROOT/subprojects/magpie/platform-linux/hwdb/hw.db"
export MC_RESOURCE_DIR="$BUILD_ROOT/resources"

# Compile GSettings schemas
glib-compile-schemas --strict "$(pwd)/data" && \
    mv "$(pwd)/data/gschemas.compiled" "$BUILD_ROOT/data/"
```

Run the application:

```bash
"$BUILD_ROOT/src/missioncenter"
```

## 📥 Installing System-wide

```bash
sudo ninja -C "$BUILD_ROOT" install
```

Then launch from your application menu or run:

```bash
missioncenter
```

## 🐛 Troubleshooting

### Character images not showing

- Verify images are in `resources/characters/`
- Check they are registered in `resources/pukupuku.gresource.xml`
- Rebuild: `ninja -C "$BUILD_ROOT"`

### GSettings schema errors

```bash
# Recompile schemas
glib-compile-schemas --strict "$(pwd)/data"
mv "$(pwd)/data/gschemas.compiled" "$BUILD_ROOT/data/"
```

### Blueprint compile errors

```bash
# Check Blueprint compiler is installed
python3 -m pip install --user blueprint-compiler
```

### Magpie not found

```bash
# Ensure magpie is in PATH
export PATH="$BUILD_ROOT/subprojects/magpie/src:$PATH"

# Check it exists
ls -la "$BUILD_ROOT/subprojects/magpie/src/magpie"
```

## 🧹 Cleaning

```bash
# Remove build directory
rm -rf "$BUILD_ROOT"

# Or use meson
meson setup --wipe "$BUILD_ROOT"
```

## 🔄 Rebuilding

```bash
# Reconfigure and rebuild
meson setup --reconfigure "$BUILD_ROOT"
ninja -C "$BUILD_ROOT"
```

## 📝 Development Tips

### Quick iteration

```bash
# Only rebuild changed files
ninja -C "$BUILD_ROOT"

# Run immediately
"$BUILD_ROOT/src/missioncenter"
```

### Check for errors

```bash
# Validate desktop file
desktop-file-validate "$BUILD_ROOT/data/io.github.rinta.PukuPuku.desktop"

# Validate schema
glib-compile-schemas --strict "$(pwd)/data"
```

### Debug logging

```bash
# Set log level
G_MESSAGES_DEBUG=all "$BUILD_ROOT/src/missioncenter"
```

## 🎨 UI Development

Blueprint files are in `resources/ui/`. After editing:

```bash
# Rebuild just the UI
ninja -C "$BUILD_ROOT" pukupuku-gresources
ninja -C "$BUILD_ROOT"
```

## ⚡ Performance

For maximum performance:

```bash
BUILD_ROOT="$(pwd)/build-meson-release"

meson setup "$BUILD_ROOT" \
    -Dbuildtype=release \
    -Db_lto=true

CC=clang CXX=clang++ ninja -C "$BUILD_ROOT"
```

Note: LTO requires clang/lld.

## 📦 Packaging

See README.md for Flatpak/AppImage/Snap instructions.

## 🤝 Contributing

When making changes:

1. Test in debug build first
2. Verify all features work
3. Check for memory leaks with valgrind
4. Update translations if needed

```bash
# Update translation template
ninja -C "$BUILD_ROOT" missioncenter-pot
```
