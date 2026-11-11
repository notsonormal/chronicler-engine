"""Test E: converter round trip. grid text -> indexed PNG (what an editor opens) -> grid text. Must be identical."""
from PIL import Image
from grid24 import compose, PAL
rows = compose('happy')
keys = list(PAL)                       # fixed palette order; index 0 = '.' transparent
im = Image.new('P', (24, 24), 0)
flat = []
for k in keys:
    c = PAL[k] or '#000000'
    flat += [int(c[i:i+2], 16) for i in (1, 3, 5)]
im.putpalette(flat + [0] * (768 - len(flat)))
for y, r in enumerate(rows):
    for x, ch in enumerate(r): im.putpixel((x, y), keys.index(ch))
im.info['transparency'] = 0
im.save('happy_indexed.png', transparency=0)
back = Image.open('happy_indexed.png')
pal = back.getpalette()
rgb2key = {tuple(pal[i*3:i*3+3]): k for i, k in enumerate(keys) if PAL[k]}
out = []
for y in range(24):
    line = ''
    for x in range(24):
        i = back.getpixel((x, y))
        line += '.' if i == 0 else keys[i]
    out.append(line)
import os
print('identical:', out == rows, '| png bytes:', os.path.getsize('happy_indexed.png'), '| grid text bytes:', sum(len(r) + 1 for r in rows))
# a second path: an RGBA PNG saved by an editor with no fixed palette order -> match by colour
rgba = Image.open('happy_indexed.png').convert('RGBA'); rgba.save('happy_rgba.png')
r2 = Image.open('happy_rgba.png'); col2key = {tuple(int(PAL[k][i:i+2], 16) for i in (1,3,5)) + (255,): k for k in PAL if PAL[k]}
out2 = [''.join('.' if r2.getpixel((x, y))[3] == 0 else col2key[r2.getpixel((x, y))] for x in range(24)) for y in range(24)]
print('rgba path identical:', out2 == rows)
