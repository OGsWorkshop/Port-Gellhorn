# Roadmap: 30 fps on a mid-range NVIDIA GPU

Current state is in [STATUS.md](STATUS.md). The target is 30 fps in the open world on a card such as an RTX 3060, with a correct final image. Items are ordered roughly by what blocks the next one; none has a date.

## 1. A correct final image

The final image is black on both runtimes because some lighting shaders fail to translate on NVIDIA and their work is dropped. Nothing below can be judged by eye until this is fixed. Remaining untranslatable groups (counts from one VIctory world run, see STATUS):

- **Descriptor read as data** (149 shaders): a resource descriptor is loaded like ordinary memory and used as a value. DXIL has no way to read a descriptor's bits; needs a mapping from descriptor contents to a table index.
- **Cross-lane ops in divergent flow** (57): wave operations inside non-uniform control flow, where the active-lane set matters.
- **Scalar memory as data** (part of the 101 `v_readfirstlane` and structured-buffer cases): scalar loads whose result is used as plain data rather than as an address or descriptor. The uniform `v_readfirstlane` case is fixed; the divergent one is not.
- **Structured buffer of unknown stride** (63).

## 2. Series S profile

Run the title the way the smaller console does: its memory budget, resolution and feature set, instead of the larger console's. This lowers memory (XCL reaches ~29 GB committed at world start) and shader and fill cost. Open question: which settings the title reads from the platform, and which can be forced.

## 3. Ahead-of-time shader and pipeline build

First-run DXC compiles and pipeline creation stall the world. The journal and offline build in XCL record the pipelines a run needs and compile them before play; the aim is a cache warm-up that finishes before the first frame (task #9 in the README) and ships as a script or a launcher step. The VIctory stack needs the same journal.

## 4. Per-frame waits and copies

Title memory copies cost up to ~600 ms per frame in world scenes (README #8). Options: skip clean pages, batch uploads, use a copy queue and overlap it with rendering. Also audit CPU/GPU fences that serialize the frame.

## 5. Shader efficiency

- **EXEC elision**: do not emit lane-mask handling where the mask is provably all-active.
- **Wave64 on 32-lane GPUs**: NVIDIA runs 32-lane waves, so wave64 code is emulated (`VICTORY_WAVE64_EMULATE`). Emulation costs time; avoid it where a workgroup does not depend on 64-lane semantics. Compute groups above 64 threads and whole-wave sections are still unsupported.
- Reduce redundant resource loads and descriptor-heap indirection in generated code.

## 6. Ray tracing mode

- **Off** (current): ray queries return no hit.
- **On** (future): map the console's ray-tracing structures to DXR acceleration structures and `TraceRay`/inline ray queries. Needs an acceleration-structure build path and a mapping for the title's traversal data. Not started.

## 7. Memory

Reduce committed memory at world start, handle NVIDIA's 1,000,000-descriptor heap cap (capped to 868,927 slots in the VIctory stack) and keep streaming within the VRAM of a 12 GB card or smaller.
