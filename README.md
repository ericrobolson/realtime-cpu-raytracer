CPU based 3d ray tracing engine
* Not production ready in any way. Simply a fun experiment to get some OpenGL sprites drawing.
* Code is loosely organized by `crates` containing engine code and `src` containing application specific code.
* Various settings can be changed in `main.rs`, though it is not very ergonomic.

Features
* Dynamic resolution scaling
* OpenGL rasterizing
* CPU driven

ARCHITECTURE
* `src` - The ray tracing specific code. `main.rs` allows tweaking of files. `renderer.rs` communicates with the hardware renderer, `core_raytracer` contains raytracing specific code. `profiling.rs` allows profiling to be executed.
* `crates` - Engine specific code. Nothing game related lives in here.


Example
* ![render](https://user-images.githubusercontent.com/9857732/111369760-ab0c0a80-8654-11eb-961d-6084deae31fc.png)
