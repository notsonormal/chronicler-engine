"""Test D: vector SVG (curves) in a 64x64 viewBox. One <g> per Expression part; the page shows each at 3 sizes.
Also: grid24 -> SVG rect runs (lossless converter) in the last row."""
from grid24 import BASE, PATCH, PAL, compose

def face_svg(expr, size):
    eyes = {
     'neutral':  '<ellipse cx="24" cy="30" rx="3.2" ry="3.6" fill="#fff" stroke="#2b1d2e" stroke-width="1"/><ellipse cx="40" cy="30" rx="3.2" ry="3.6" fill="#fff" stroke="#2b1d2e" stroke-width="1"/><circle cx="24.4" cy="30.4" r="1.9" fill="#4a7fb0"/><circle cx="40.4" cy="30.4" r="1.9" fill="#4a7fb0"/><circle cx="24.4" cy="30.4" r="0.9" fill="#2b1d2e"/><circle cx="40.4" cy="30.4" r="0.9" fill="#2b1d2e"/>',
     'happy':    '<path d="M20.5 31 Q24 26 27.5 31" fill="none" stroke="#2b1d2e" stroke-width="1.8" stroke-linecap="round"/><path d="M36.5 31 Q40 26 43.5 31" fill="none" stroke="#2b1d2e" stroke-width="1.8" stroke-linecap="round"/>',
     'angry':    '<ellipse cx="24" cy="31" rx="3.2" ry="2.6" fill="#fff" stroke="#2b1d2e" stroke-width="1"/><ellipse cx="40" cy="31" rx="3.2" ry="2.6" fill="#fff" stroke="#2b1d2e" stroke-width="1"/><circle cx="24.4" cy="31.4" r="1.5" fill="#2b1d2e"/><circle cx="40.4" cy="31.4" r="1.5" fill="#2b1d2e"/>',
     'surprised':'<ellipse cx="24" cy="30" rx="4.2" ry="5" fill="#fff" stroke="#2b1d2e" stroke-width="1"/><ellipse cx="40" cy="30" rx="4.2" ry="5" fill="#fff" stroke="#2b1d2e" stroke-width="1"/><circle cx="24" cy="30" r="1.3" fill="#2b1d2e"/><circle cx="40" cy="30" r="1.3" fill="#2b1d2e"/>',
     'blink':    '<path d="M20.5 30.5 Q24 33 27.5 30.5" fill="none" stroke="#2b1d2e" stroke-width="1.8" stroke-linecap="round"/><path d="M36.5 30.5 Q40 33 43.5 30.5" fill="none" stroke="#2b1d2e" stroke-width="1.8" stroke-linecap="round"/>',
    }[expr]
    brows = {
     'neutral':  '<path d="M19 24 L28 23.5 M36 23.5 L45 24" stroke="#5a3a2a" stroke-width="2" stroke-linecap="round"/>',
     'happy':    '<path d="M19 23.5 Q23.5 21 28 23" stroke="#5a3a2a" stroke-width="2" fill="none" stroke-linecap="round"/><path d="M36 23 Q40.5 21 45 23.5" stroke="#5a3a2a" stroke-width="2" fill="none" stroke-linecap="round"/>',
     'angry':    '<path d="M19 22 L28 26 M36 26 L45 22" stroke="#5a3a2a" stroke-width="2.2" stroke-linecap="round"/>',
     'surprised':'<path d="M19 20 Q23.5 17.5 28 20 M36 20 Q40.5 17.5 45 20" stroke="#5a3a2a" stroke-width="2" fill="none" stroke-linecap="round"/>',
     'blink':    '<path d="M19 24 L28 23.5 M36 23.5 L45 24" stroke="#5a3a2a" stroke-width="2" stroke-linecap="round"/>',
    }[expr]
    mouth = {
     'neutral':  '<path d="M27 43 L37 43" stroke="#9b3a3a" stroke-width="1.6" stroke-linecap="round"/>',
     'happy':    '<path d="M25 41 Q32 50 39 41 Z" fill="#9b3a3a" stroke="#2b1d2e" stroke-width="1" stroke-linejoin="round"/><circle cx="19" cy="38" r="2.6" fill="#e8907f" opacity=".7"/><circle cx="45" cy="38" r="2.6" fill="#e8907f" opacity=".7"/>',
     'angry':    '<path d="M26 45 Q32 40 38 45" stroke="#9b3a3a" stroke-width="1.6" fill="none" stroke-linecap="round"/>',
     'surprised':'<ellipse cx="32" cy="44" rx="3" ry="4" fill="#9b3a3a" stroke="#2b1d2e" stroke-width="1"/>',
     'blink':    '<path d="M27 43 L37 43" stroke="#9b3a3a" stroke-width="1.6" stroke-linecap="round"/>',
    }[expr]
    return f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" width="{size}" height="{size}">
<path d="M8 64 Q10 50 24 48 L40 48 Q54 50 56 64 Z" fill="#3a5f9b" stroke="#2b1d2e" stroke-width="1.2"/>
<rect x="27" y="44" width="10" height="8" fill="#d49a76"/>
<ellipse cx="32" cy="30" rx="16" ry="19" fill="#f0c29a" stroke="#2b1d2e" stroke-width="1.4"/>
<path d="M15 28 Q13 8 32 7 Q51 8 49 28 Q47 18 40 17 Q32 22 24 17 Q17 18 15 28 Z" fill="#5a3a2a" stroke="#2b1d2e" stroke-width="1.2" stroke-linejoin="round"/>
<path d="M31 33 L30 37 L33 37" fill="none" stroke="#d49a76" stroke-width="1.2" stroke-linecap="round"/>
{brows}{eyes}{mouth}</svg>'''

def grid_svg(rows, size):
    """Lossless grid -> SVG: one <rect> per horizontal run of one colour."""
    out = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {len(rows[0])} {len(rows)}" width="{size}" height="{size}" shape-rendering="crispEdges">']
    n = 0
    for y, r in enumerate(rows):
        x = 0
        while x < len(r):
            ch = r[x]; j = x
            while j < len(r) and r[j] == ch: j += 1
            if PAL[ch]: out.append(f'<rect x="{x}" y="{y}" width="{j-x}" height="1" fill="{PAL[ch]}"/>'); n += 1
            x = j
    out.append('</svg>')
    return '\n'.join(out), n

names = ['neutral','happy','angry','surprised','blink']
html = ['<html><body style="background:#cfd8dc;margin:10px;font:12px sans-serif">']
for size in (32, 64, 192):
    html.append('<div style="display:flex;gap:10px;align-items:end;margin-bottom:6px">' + ''.join(face_svg(n, size) for n in names) + '</div>')
html.append('<div style="display:flex;gap:10px">')
total = 0
for n in names:
    s, c = grid_svg(compose(n), 192); html.append(s); total += c
html.append('</div><p>grid24 as SVG rects: avg %d rects per Expression</p></body></html>' % (total // len(names)))
open('svg_test.html', 'w').write('\n'.join(html))
open('svg_face_neutral.svg', 'w').write(face_svg('neutral', 64))
print(len(face_svg('neutral', 64)), 'bytes neutral svg;', grid_svg(compose('neutral'), 24)[1], 'rects')
