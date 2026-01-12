# PukuPuku

> **Note:** PukuPuku is based on [Mission Center](https://gitlab.com/mission-center-devs/mission-center) and extends it with a hand-drawn, character-driven interface.

A cute and casual task manager + benchmark app for Linux!

かわいいLinux向けタスクマネージャー+ベンチマークアプリ

## Features

### System Monitoring
* Monitor CPU, Memory, Disk, Network and GPU usage
* Casual and lightweight system status display
* Character-based status visualization (3 mood states)
  - Normal: Your PC is doing fine
  - Very Hard: Working hard
  - Danger: Might be in trouble

### Benchmarking
* CPU performance benchmark
* I/O speed benchmark (disk read/write)
* Optional CPU-only benchmark mode for quick testing
* Quick and lightweight performance testing

### Advanced Monitoring
* Monitor overall or per-thread CPU usage
* RAM and Swap usage tracking
* Disk utilization and transfer rates
* Network utilization and transfer speeds
* GPU usage, memory, and power consumption (via NVTOP)
* Per-app and per-process resource breakdown

### Technology
* Built with GTK4 and Libadwaita
* Written in Rust for performance and safety
* Hardware-accelerated graph rendering
* Based on Mission Center architecture
* GNOME 49 runtime support

## Design Philosophy

PukuPuku aims to make system monitoring fun and accessible:
- Casual: No intimidating technical jargon
- Lightweight: Quick to launch and easy on resources
- Visual: Character expressions show your PC's mood

## Installation

### Building from Source

**Requirements:**

| Dependency                   | Minimum Version |
|------------------------------|----------------:|
| Meson                        |           1.0.2 |
| Rust                         |            1.90 |
| CMake                        |            3.15 |
| Python3                      |            3.10 |
| Python GObject Introspection |             N/A |
| DRM development libraries    |             N/A |
| GBM development libraries    |             N/A |
| udev development libraries   |             N/A |
| GTK 4                        |            4.20 |
| libadwaita                   |             1.8 |

> Note: After cloning, run this to fetch required subprojects (such as magpie).
```
git submodule update --init --recursive
```


**Build instructions:**

```bash
BUILD_ROOT="$(pwd)/build-meson-debug"

meson setup "$BUILD_ROOT" -Dbuildtype=debug
ninja -C "$BUILD_ROOT"
```

**Running:**

```bash
export PATH="$BUILD_ROOT/subprojects/magpie/src:$PATH"
export GSETTINGS_SCHEMA_DIR="$BUILD_ROOT/data"
export MC_MAGPIE_HW_DB="$BUILD_ROOT/subprojects/magpie/platform-linux/hwdb/hw.db"
export MC_RESOURCE_DIR="$BUILD_ROOT/resources"

glib-compile-schemas --strict "$(pwd)/data" && mv "$(pwd)/data/gschemas.compiled" "$BUILD_ROOT/data/"

"$BUILD_ROOT/src/missioncenter"
```

**Installation:**

```bash
ninja -C $BUILD_ROOT install
```

## Character Images

Place your character images in `resources/characters/`:
- `normal.png` - Your PC is doing fine
- `very_hard.png` - Working hard
- `danger.png` - Might be in trouble

The character's mood changes based on:
- CPU usage (60%+ = Very Hard, 90%+ = Danger)
- Memory usage (75%+ = Very Hard, 90%+ = Danger)
- Disk usage (75%+ = Very Hard, 90%+ = Danger)

Hand-drawn black and white images work best with PukuPuku's aesthetic.

## Credits

PukuPuku is based on Mission Center by Mission Center Developers.

The original Mission Center architecture provides robust system monitoring capabilities,
which PukuPuku extends with a casual, character-driven interface and hand-drawn aesthetic.

## License

GPL-3.0-or-later

## Acknowledgments

- Mission Center for the excellent monitoring foundation
- GTK and GNOME for the UI framework
- NVTOP for GPU monitoring support
- Rust Community for the amazing tooling

## Flatpak Installation

Install the required Flatpak runtimes and SDKs:

```bash
flatpak install -y \
    org.freedesktop.Platform//25.08 \
    org.freedesktop.Sdk//25.08 \
    org.gnome.Platform//49 \
    org.gnome.Sdk//49
```

Build a Flatpak package:

```bash
cd flatpak
flatpak-builder --user --install --force-clean build-flatpak io.missioncenter.MissionCenter.json
```

Run the app from your launcher or from the command-line:

```bash
flatpak run io.github.rinta.PukuPuku
```


