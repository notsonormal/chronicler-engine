"""Test C: drawing code. Shape primitives (rect, ellipse, polygon, line) on a 48x48 indexed canvas."""
from PIL import Image, ImageDraw
PAL = {0: (0,0,0,0), 1: '#2b1d2e', 2: '#f0c29a', 3: '#d49a76', 4: '#5a3a2a', 5: '#8a6046', 6: '#ffffff',
       7: '#4a7fb0', 8: '#9b3a3a', 9: '#3a5f9b', 10: '#2c4775', 11: '#e8907f'}
N = 48

def canvas():
    return Image.new('P', (N, N), 0)

def draw(expr):
    im = canvas(); d = ImageDraw.Draw(im)
    pal = []
    for i in range(12):
        c = PAL[i]
        if isinstance(c, str): c = tuple(int(c[j:j+2], 16) for j in (1,3,5))
        pal += list(c[:3])
    im.putpalette(pal + [0]*(768-len(pal)))
    # body
    d.polygon([(6,47),(8,38),(16,33),(32,33),(40,38),(42,47)], fill=9, outline=1)
    d.rectangle([20,28,28,36], fill=3, outline=1)
    d.polygon([(19,35),(24,40),(29,35),(24,34)], fill=10)
    # head
    d.ellipse([10,5,38,34], fill=2, outline=1)
    # hair
    d.pieslice([8,2,40,32], 180, 360, fill=4, outline=1)
    d.polygon([(10,16),(12,10),(18,12),(24,8),(30,12),(36,10),(38,16),(38,10),(34,5),(14,5)], fill=4)
    d.rectangle([9,12,12,20], fill=4); d.rectangle([36,12,39,20], fill=4)
    d.line([(18,8),(22,9)], fill=5); d.line([(26,7),(30,8)], fill=5)
    # nose
    d.rectangle([23,23,24,25], fill=3)
    def eye(cx, kind):
        if kind == 'open':
            d.rectangle([cx-3,18,cx+3,22], fill=6); d.rectangle([cx-1,18,cx+1,22], fill=7); d.rectangle([cx,19,cx,21], fill=1)
            d.line([(cx-3,17),(cx+3,17)], fill=1)
        elif kind == 'wide':
            d.rectangle([cx-3,17,cx+3,23], fill=6); d.rectangle([cx-1,19,cx+1,21], fill=7); d.point((cx,20), fill=1)
        elif kind == 'happy':
            d.line([(cx-3,21),(cx-1,18),(cx+1,18),(cx+3,21)], fill=1)
        elif kind == 'closed':
            d.line([(cx-3,20),(cx+3,20)], fill=1)
        elif kind == 'angry':
            d.rectangle([cx-3,19,cx+3,22], fill=6); d.rectangle([cx-1,20,cx+1,22], fill=7); d.point((cx,21), fill=1)
    def brow(cx, tilt):  # tilt: -1 inner-down (angry), 0 flat, +1 raised
        if tilt == 0:  d.line([(cx-4,15),(cx+4,15)], fill=4, width=1)
        if tilt == 1:  d.line([(cx-4,13),(cx+4,13)], fill=4, width=1)
        if tilt == -1:
            d.line([(cx-4,14),(cx+4,17)] if cx < 24 else [(cx-4,17),(cx+4,14)], fill=4, width=1)
    E = {'neutral': ('open',0), 'happy': ('happy',0), 'angry': ('angry',-1), 'surprised': ('wide',1), 'blink': ('closed',0)}[expr]
    for cx in (17, 31):
        eye(cx, E[0]); brow(cx, E[1])
    # mouth
    if expr == 'neutral': d.line([(20,29),(28,29)], fill=8, width=1)
    if expr == 'blink':   d.line([(20,29),(28,29)], fill=8, width=1)
    if expr == 'happy':
        d.chord([19,26,29,32], 0, 180, fill=8, outline=1); d.rectangle([14,26,16,27], fill=11); d.rectangle([32,26,34,27], fill=11)
    if expr == 'angry':   d.line([(19,30),(22,28),(26,28),(29,30)], fill=8)
    if expr == 'surprised': d.ellipse([22,27,26,32], fill=8, outline=1)
    im.info['transparency'] = 0
    return im.convert('RGBA')

if __name__ == '__main__':
    names = ['neutral','happy','angry','surprised','blink']
    sheet = Image.new('RGBA', (N*6*len(names)+10*(len(names)+1), N*6+20), '#cfd8dc')
    for i, n in enumerate(names):
        sheet.alpha_composite(draw(n).resize((N*6,N*6), Image.NEAREST), (10+i*(N*6+10), 10))
    sheet.save('prims48_attempt2.png')
