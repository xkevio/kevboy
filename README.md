## egui + eframe with `--release`

Tetris is the baseline test. No VSync and APU is disabled.
Next optimization will include the previous one unless otherwise specified.

- Baseline: ~280fps
- For `read` + `write`, only match on top 4 bits of the address: ~295fps
- Precompute DMG palette for background + win: ~310fps
- Precompute DMG palette for sprites: ~310fps
- Skip enum dispatch of MBC type and force MBC0: ~310-315fps *(might do more for more cartridge-heavy games)*
- *(ignore prev. opt)*; Templating / compile time cgb checks in PPU: ~320-330fps
- Moving a `Vec` instead of taking a reference (not needed afterwards, `Vec::from` slow): ~380-400fps
- Skip debug background map UI: ~450fps
- Remove unnecessary conversion of `Vec` of tuples to one of its members: ~500-520fps

Some other stuff: ~600fps

#### Profiling with `debug = true` after those opts applied:
![image](https://user-images.githubusercontent.com/13004777/254681744-17b35237-f228-4734-9e5c-57b9833c9b1f.png)


