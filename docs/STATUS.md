# Technical status

Reference machine: Windows 10 22H2, RTX 3060 12 GB, 32 GB RAM.

## What works
- Executable maps, all imports bind (0 missing), the game's console DLLs load and run.
- Intro, first-boot setup and menus at 30 fps (XCL); audio plays. No Xbox Live: sign-in fails cleanly and the game carries on offline.
- World packs stream in through DirectStorage; the open world loads.
- VIctory stack: world geometry drawn live (debug colour view, about 1 fps) and the movement tutorial is reached. Exposure, luminance and tonemap shaders (36) translate.
- XCL: world-start GPU crash fixed (the game survives 7+ minutes in the world), pipeline journal and offline build, launcher.

## What does not
- The final image is black on both runtimes: some lighting shaders still fail to translate on NVIDIA and their work is dropped.
- VIctory is about 1 fps in the world; title memory copies cost up to ~600 ms per frame.
- XCL: first-run shader compiles stall the world; two-pass mesh draws (`XCL_TWO_PASS_INDIRECT=1`) can reset the GPU; memory reaches ~29 GB committed at world start.

## Shader failure groups (VIctory, one world run)
| Failure | Shaders |
|---|:---:|
| Descriptor read only as data | 149 |
| `v_readfirstlane` in divergent control flow | 101 (uniform case fixed) |
| Lane mask read as data (mesh and compute) | 94 (fixed in our stack) |
| Structured buffer of unknown stride | 63 |
| Cross-lane wave op in divergent flow | 57 |
| Title heap above NVIDIA's 1,000,000-descriptor cap | capped to 868,927 slots in our stack |

## Shader coverage
- Pack: ~26,500 containers / 26,084 programs. XCL translates all to HLSL; DXC compiles them in the background.
- VIctory (integration build): DXIL for 14,136/14,208 VS/PS, 2,261/2,582 CS, 8,365/8,520 MS on a 32-lane host.

## Known gaps
- Mesh pipelines with a real AS + MS pair.
- On 32-lane GPUs, wave64 compute groups larger than 64 threads and whole-wave sections.
