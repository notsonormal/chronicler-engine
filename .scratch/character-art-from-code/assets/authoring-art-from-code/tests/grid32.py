"""Test B: raw 32x32 text grid, one frame per Expression (rows typed by hand, centred by a helper)."""
from PIL import Image
from grid24 import PAL as P24, render
PAL = dict(P24); PAL.update({'i': '#4a7fb0'})
W = 32
def centre(core):
    assert (W - len(core)) % 2 == 0, (len(core), core)
    p = (W - len(core)) // 2
    return '.' * p + core + '.' * p
def I(*parts):
    s = ''.join(parts); assert len(s) == 18, (len(s), s); return 'o' + s + 'o'
sk = "s" * 18
def rows(face):   # face = dict row-> core override for rows 10..19
    r = ["", "",
     "oooooooooo", "ohhhHHhhhhho", "ohhhhHHhhhhhhhho", "o"+"h"*16+"o", "ohhhhhhHHhhhhhhhho", "o"+"h"*18+"o",
     "ohhhhsssshhsssshhhho", "o"+"hh"+"s"*14+"hh"+"o"]
    for y in range(10, 20): r.append(face[y])
    r += ["o"+"s"*16+"o", "o"+"s"*14+"o", "o"+"s"*12+"o", "o"+"s"*10+"o",
          "oSSSSSSSSo", "oSSSSSSSSo", "occccSSSSSSSScccco", "o"+"c"*24+"o"]
    r += ["o"+"c"*28+"o"] * 4
    return [centre(c) if c else '.' * W for c in r]
neutral = {10: I("ss","hhhhh","ssss","hhhhh","ss"), 11: I(sk), 12: I("sss","eeee","ssss","eeee","sss"),
           13: I("sss","wiiw","ssss","wiiw","sss"), 14: I("sss","wiiw","ssss","wiiw","sss"),
           15: I("s"*8,"SS","s"*8), 16: I("s"*8,"SS","s"*8), 17: I(sk), 18: I("s"*6,"mmmmmm","s"*6), 19: I(sk)}
happy = dict(neutral)
happy.update({12: I("sss","sees","ssss","sees","sss"), 13: I("sss","esse","ssss","esse","sss"),
              14: I(sk), 18: I("s"*5,"m","ssssss","m","s"*5), 19: I("s"*6,"mmmmmm","s"*6)})
for name, face in (("neutral", neutral), ("happy", happy)):
    render(rows(face), 12, PAL).save(f"grid32_{name}.png")
open("grid32_neutral.txt", "w").write("\n".join(rows(neutral)))
