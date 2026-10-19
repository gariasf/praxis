# PR images

Before, after and difference images that pull request descriptions link
to, one directory per pull request (`docs/agents/workflow.md`, PRs). This
branch is never merged; descriptions link its commits by SHA:
`https://raw.githubusercontent.com/gariasf/praxis/<commit>/<pr>/<file>.png`.

- Before: main's Golden (`git show <main>:tests/goldens/<key>/<scene>.png`)
  or a capture from a main build; after: the branch's Golden or
  `praxis capture <scene> <png> 640x360`.
- `python3 -I difference.py before.png after.png difference.png [tolerance]`
  writes the Goldens test's difference image: the after image dimmed to a
  third, every pixel more than `tolerance` (default 1) steps off in red.
  It prints how many pixels changed. `magick compare` miscounts RGBA PNGs.
- Zoom: `magick in.png -alpha off -crop WxH+X+Y +repage -filter point
  -resize 400% out.png`, then `magick before.png after.png +append`.
