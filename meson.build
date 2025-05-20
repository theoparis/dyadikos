project(
    'dyadikos',
    'c',
    default_options: [
        'c_std=c2x',
        'warning_level=3',
        'b_lto=true',
        'buildtype=debugoptimized',
    ],
)

inc = include_directories('src')

# dependencies
dep_cglm = dependency('cglm', fallback: ['cglm', 'cglm_dep'])
# dep_flecs = dependency('flecs', fallback: ['flecs', 'flecs_dep'])
dep_vk = dependency('vulkan')
dep_xcb = dependency('xcb')

libdyadikos = library(
    'dyadikos',
    'src/clay.c',
    'src/window.c',
    dependencies: [dep_cglm, dep_vk, dep_xcb],
    install: true,
    include_directories: inc,
    c_args: ['-D_DEFAULT_SOURCE'],
)
libdyadikos_dep = declare_dependency(
    include_directories: inc,
    link_with: libdyadikos,
)

executable(
    'dyadikos',
    sources: ['src/main.c'],
    dependencies: [libdyadikos_dep],
    install: true,
)
