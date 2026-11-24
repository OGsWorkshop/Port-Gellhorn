# Technical status

Reference machine: Windows 10 22H2, RTX 3060 12 GB, 32 GB RAM.

## Boot → menus (works)
- Executable maps, all imports bind (0 missing), and the game's console DLLs (Bink 2, BVH builder, FSR loader, libHttpClient) load and run.
- Intro, first-boot setup and menus present at a locked 30 fps; audio plays.
- No Xbox Live: sign-in calls fail cleanly (`0x80072EFD`) and the game carries on offline.

## Shaders
- Pack: ~26,500 containers / 26,084 programs.
- XCL translates all of them to HLSL; DXC compiles them in the background (stand-in pipelines draw nothing until then).
- VIctory (integration build) emits DXIL for 14,136/14,208 VS/PS, 2,261/2,582 CS and 8,365/8,520 MS on a 32-lane host.
- All 36 exposure, luminance and tonemap shaders translate.

## World (the current wall)
- World packs (`core_restofworld*.rpf`) stream in through DirectStorage; draws and dispatches run with 0 skipped.
- Output stays black, for two reasons:
  1. First-run shader compiles stall rendering long enough for the game's hang detection to quit.
  2. Cross-queue GPU waits are resolved in the wrong order, so draws read memory before it is written.
- Console mesh pipelines put their mesh shader in the AS slot. Upstream XCL refuses them; our patch translates them (on trial).

## Known gaps
- Mesh pipelines with a real AS + MS pair: not supported yet.
- On 32-lane GPUs, wave64 compute groups larger than 64 threads, and whole-wave sections, don't translate in VIctory.
- In world scenes, title memory copies cost up to ~600 ms/frame in VIctory.
