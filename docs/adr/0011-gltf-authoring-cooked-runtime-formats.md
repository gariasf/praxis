# glTF for authoring, cooked binaries at runtime

glTF (from Blender) is the authoring source; the runtime loads cooked engine-native binaries with mip chains, GPU block compression (BC) and streaming-sized chunks. Loading glTF at runtime stays a development convenience. Cooking is core engine work, not a late tool: it is the main memory lever (the material arrays are 640 MiB of uncompressed RGBA8 without mips today) and it defines what streaming can load. Accepted 2026-10-12.
