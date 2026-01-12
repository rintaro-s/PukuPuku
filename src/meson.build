pkgdatadir = join_paths(get_option('prefix'), get_option('datadir'), meson.project_name())

gnome = import('gnome')
libudev = dependency('libudev', required: true)

conf = configuration_data()
conf.set_quoted('VERSION', meson.project_version())
conf.set_quoted('GETTEXT_PACKAGE', 'missioncenter')
conf.set_quoted('LOCALEDIR', join_paths(get_option('prefix'), get_option('localedir')))
conf.set_quoted('PKGDATADIR', pkgdatadir)

configure_file(
    input: 'config.rs.in',
    output: 'config.rs',
    configuration: conf
)

cargo_bin = find_program('cargo')
cargo_env = get_option('cargo_env')
rustflags = []
if get_option('flatpak') == false
    cargo_env += [ 'CARGO_HOME=' + meson.project_build_root()  / 'cargo-home' ]
    cargo_opt  = []
else
    cargo_opt  = [ '--features=skip-codegen' ]
endif

if get_option('skip-codegen')
    cargo_opt += [ '--features=skip-codegen' ]
endif

cargo_opt += [ '--manifest-path', meson.project_source_root() / 'Cargo.toml' ]
cargo_opt += [ '--target-dir', meson.project_build_root()  / 'src' ]
cargo_env += [ 'BUILD_ROOT=' + meson.project_build_root() ]

if get_option('b_lto') == true
    if c_compiler.get_id() != 'clang'
        error('LTO is only supported with CLang. Make sure both C and C++ compilers are CLang')
    endif

    if c_compiler.get_linker_id() != 'ld.lld'
        error('LTO is only supported with lld')
    endif

    cargo_env += ['CC=clang']
    cargo_env += ['CXX=clang++']
    cargo_env += ['CFLAGS="-fuse-ld=lld -flto"']
    cargo_env += ['CXXFLAGS="-fuse-ld=lld -flto"']

    rustflags += ['-Clinker=clang', '-Clink-arg=-fuse-ld=lld', '-Clinker-plugin-lto']
endif

cargo_env += ['RUSTFLAGS="' + ' '.join(rustflags) + '"']

if get_option('buildtype') == 'release'
  cargo_opt += [ '--release' ]
  rust_target = 'release'
else
  rust_target = 'debug'
endif

cargo_build = custom_target(
  'cargo-build',
  build_by_default: true,
  build_always_stale: true,
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: get_option('bindir'),
  command: [
    'sh', '-c', 'env '
    + ' env '.join(cargo_env)
    + ' ' + cargo_bin.full_path() + ' build ' + ' '.join(cargo_opt)
    + ' ' + '&&'
    + ' ' + 'cp ' + meson.project_build_root() + '/src/' + rust_target + '/' + meson.project_name() + ' ' + '@OUTPUT@'
  ]
)
