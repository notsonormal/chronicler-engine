"""Test A: text grid + palette, 24x24 bust. Base layer + one face patch per Expression (rows 9..14)."""
from PIL import Image

PAL = {'.': None, 'o': '#2b1d2e', 's': '#f0c29a', 'S': '#d49a76', 'h': '#5a3a2a', 'H': '#8a6046',
       'w': '#ffffff', 'e': '#2b1d2e', 'm': '#9b3a3a', 'c': '#3a5f9b', 'C': '#2c4775', 'r': '#e8907f'}
W = 24

def centre(core):
    assert (W - len(core)) % 2 == 0, (len(core), core)
    p = (W - len(core)) // 2
    return '.' * p + core + '.' * p

face = "o" + "s" * 14 + "o"
BASE = [centre(c) if c else '.' * W for c in [
    "",
    "oooooooooo",
    "ohhHHhhhhhho",
    "ohhhhHHhhhhhho",
    "ohhhhhhhhhhhhhho",
    "ohhhhhhhhhhhhhho",
    "ohhhhhhhhhhhhhho",
    "ohhhssshhssshhho",
    "ohssssssssssssho",
    face, face, face, face, face, face,
    "o" + "s" * 12 + "o",
    "o" + "s" * 10 + "o",
    "o" + "s" * 8 + "o",
    "oSSSSSSo",
    "ocCSSSSSSCcco"[:0] + "occCSSSSSSCcco",   # 14 wide collar row
    "o" + "c" * 18 + "o",
    "o" + "c" * 20 + "o",
    "o" + "c" * 20 + "o",
    "o" + "c" * 20 + "o",
]]
assert len(BASE) == 24 and all(len(r) == W for r in BASE), [len(r) for r in BASE]

# Patches replace the 14 interior columns of rows 9..14 (the face).
def L(*parts): 
    s = ''.join(parts); assert len(s) == 14, (len(s), s); return s
brow  = lambda: L("ss", "hhh", "ssss", "hhh", "ss")
none_ = lambda: L("s" * 14)
PATCH = {
 'neutral':   [brow(), none_(), L("ss","wew","ssss","wew","ss"), L("ss","wew","ssss","wew","ss"), none_(), L("sssss","mmmm","sssss")],
 'happy':     [brow(), none_(), L("ss","ses","ssss","ses","ss"), L("ss","ese","ssss","ese","ss"),
               L("s","rr","s","m","ssss","m","s","rr","s"), L("sssss","mmmm","sssss")],
 'angry':     [L("ss","hss","ssss","ssh","ss"),
               L("ss","shh","ssss","hhs","ss"), L("ss","see","ssss","ees","ss"), L("ss","swe","ssss","ews","ss"), none_(), L("ssss","mmmmmm","ssss")],
 'surprised': [brow(), none_(), L("ss","www","ssss","www","ss"), L("ss","wew","ssss","wew","ss"), L("ssssss","mm","ssssss"), L("ssssss","mm","ssssss")],
 'blink':     [brow(), none_(), none_(), L("ss","eee","ssss","eee","ss"), none_(), L("sssss","mmmm","sssss")],
}

def compose(name):
    rows = list(BASE)
    for i, p in enumerate(PATCH[name]):
        rows[9 + i] = 'o' + p + 'o'
        rows[9 + i] = ('.' * 4) + rows[9 + i] + ('.' * 4)
        assert len(rows[9 + i]) == W
    return rows

def render(rows, scale, pal=PAL):
    im = Image.new('RGBA', (len(rows[0]), len(rows)), (0, 0, 0, 0))
    for y, r in enumerate(rows):
        for x, ch in enumerate(r):
            c = pal[ch]
            if c: im.putpixel((x, y), tuple(int(c[i:i+2], 16) for i in (1, 3, 5)) + (255,))
    return im.resize((im.width * scale, im.height * scale), Image.NEAREST)

if __name__ == '__main__':
    names = list(PATCH)
    sheet = Image.new('RGBA', (24 * 8 * len(names) + 10 * (len(names) + 1), 24 * 8 + 20), '#cfd8dc')
    for i, n in enumerate(names):
        sheet.alpha_composite(render(compose(n), 8), (10 + i * (24 * 8 + 10), 10))
    sheet.save('grid24_attempt2.png')
    # also 1x for a size check
    s1 = Image.new('RGBA', (24 * 5 + 6, 24 + 4), '#cfd8dc')
    for i, n in enumerate(names):
        s1.alpha_composite(render(compose(n), 1), (2 + i * 25, 2))
    s1.resize((s1.width * 3, s1.height * 3), Image.NEAREST).save('grid24_1x_size.png')
    print('\n'.join(compose('happy')))
