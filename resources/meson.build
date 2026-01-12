pkgdatadir = join_paths(get_option('prefix'), get_option('datadir'), meson.project_name())

gnome = import('gnome')

blueprints = custom_target('blueprints',
  input: files(
    'ui/apps_page/page.blp',

    'ui/benchmark_page/page.blp',

    'ui/performance_page/cpu.blp',
    'ui/performance_page/cpu_details.blp',
    'ui/performance_page/disk.blp',
    'ui/performance_page/disk_details.blp',
    'ui/performance_page/disk_eject_failure_dialog.blp',
    'ui/performance_page/disk_eject_failure_row.blp',
    'ui/performance_page/disk_smart_data_dialog.blp',
    'ui/performance_page/disk_smart_data_entry.blp',
    'ui/performance_page/disk_smart_failure_dialog.blp',
    'ui/performance_page/fan.blp',
    'ui/performance_page/fan_details.blp',
    'ui/performance_page/gpu.blp',
    'ui/performance_page/gpu_details.blp',
    'ui/performance_page/memory.blp',
    'ui/performance_page/memory_details.blp',
    'ui/performance_page/memory_info_tooltip.blp',
    'ui/performance_page/network.blp',
    'ui/performance_page/network_details.blp',
    'ui/performance_page/page.blp',
    'ui/performance_page/summary_graph.blp',

    'ui/preferences/page.blp',
    'ui/preferences/window.blp',

    'ui/services_page/page.blp',

    'ui/table_view/table_view.blp',
    'ui/table_view/process_action_bar.blp',
    'ui/table_view/process_details_dialog.blp',
    'ui/table_view/service_action_bar.blp',
    'ui/table_view/service_details_dialog.blp',

    'ui/widgets/theme_selector.blp',
    'ui/widgets/character_status.blp',

    'ui/about_system_dialog.blp',
    'ui/keyboard_shortcuts.blp',
    'ui/window.blp',
  ),
  output: '.',
  command: [find_program('blueprint-compiler'), 'batch-compile', '@OUTPUT@', '@CURRENT_SOURCE_DIR@', '@INPUT@'],
)

gnome.compile_resources('pukupuku',
  'pukupuku.gresource.xml',
  gresource_bundle: true,
  install: true,
  install_dir: pkgdatadir,
  dependencies: blueprints,
)

