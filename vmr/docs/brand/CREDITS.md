# Brand assets

- Source photo: "pexels-badesaba-15044747" by Badesaba (Pexels License: free use and
  modification, attribution not required).
- Style: the photo is re-rendered with AnimeGANv2 (`face_paint_512_v2`, ONNX, 512×512),
  then upscaled and sharpened — same cat, anime treatment.
- `logo.png` / `mark.png` are composed from that render by `build.py` (cream card, white-ring
  medallion, "vmr" set in SF Compact Rounded, vermilion accent dot).

Rebuild either asset from any square-ish source image:

```bash
python3 build.py --src <image> --box <left,top,right,bottom>
```
