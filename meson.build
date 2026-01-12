project('missioncenter',
        'rust', 'c',
          version: '0.1.0',
    meson_version: '>= 0.63.0',
  default_options: [ 'warning_level=2', 'werror=false', ],
)

i18n = import('i18n')
gnome = import('gnome')

c_compiler = meson.get_compiler('c')

skip_codegen = get_option('skip-codegen')

if get_option('flatpak') == false
    proj_magpie = subproject('magpie', default_options: [
                                 'skip-codegen=' + skip_codegen.to_string(),
                                 'b_lto=' + get_option('b_lto').to_string(),
                             ])
endif

subdir('data')
subdir('po')
subdir('resources')
subdir('src')

gnome.post_install(
     glib_compile_schemas: true,
    gtk_update_icon_cache: true,
  update_desktop_database: true,
)
