CPU based 3d raycasting engine

Features
* Dynamic resolution scaling
* OpenGL rasterizing
* CPU driven

ARCHITECTURE
* `src` - The ray tracing specific code. `main.rs` allows tweaking of files. `renderer.rs` communicates with the hardware renderer, `core_raytracer` contains raytracing specific code. `profiling.rs` allows profiling to be executed.
* `crates` - Engine specific code. Nothing game related lives in here.