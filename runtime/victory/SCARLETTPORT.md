# Provenance

This tree is the `lane` stack used by ScarlettPort, vendored as plain source.

- Upstream: VIctory by Marvisak (https://github.com/Marvisak/VIctory), GPL-3.0-or-later. `LICENSE` is unchanged.
- Base: the AnyXboxX stable VIctory stack (https://github.com/LooperSalty/victory, branch `anyxboxx/stable`, commit 9a9eab0).
- On top of that: parts of upstream VIctory PR #994 and PR #1407, plus ScarlettPort fixes: Windows 10 Agility SDK opt-in, automatic input, press file with stick tokens, capture options, `VICTORY_SHOW_TARGET=WxH-fN` selector, dropped-command-list guard, mesh lane mask read as data, uniform `v_readfirstlane` in divergent flow, default descriptor-heap cap.
- `patches/victory-stack.patch` (repo root) is the diff of this stack against the upstream merge-base (Marvisak/VIctory `main`, 58fed37).

Build and run: see `scripts/setup-victory.ps1` and `scripts/play-victory.ps1`.
