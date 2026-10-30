"""Generate every raster art asset for Elevator Action: Red Door Protocol.

Run headlessly with Krita:

    ~/.agents/skills/krita/scripts/kritarun.sh tools/krita_assets.py

Everything is drawn as true 1x pixel art (no antialiasing) into QImages, pushed
into Krita layers and exported through Krita's projection.  The source .kra
files land in assets/art/ so the art can be edited by hand later.
"""

import ksketch as ks

ks.started()

import os
import random
import sys
import traceback

from PyQt6.QtGui import QColor
from PyQt6.QtCore import Qt

import pixelfont

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SPRITES = os.path.join(ROOT, "assets", "sprites")
BACKGROUNDS = os.path.join(ROOT, "assets", "backgrounds")
ART = os.path.join(ROOT, "assets", "art")

TRANSPARENT = QColor(0, 0, 0, 0)


def C(hexstr, a=255):
    hexstr = hexstr.lstrip("#")
    return QColor(int(hexstr[0:2], 16), int(hexstr[2:4], 16), int(hexstr[4:6], 16), a)


class Pix:
    def __init__(self, w, h):
        from PyQt6.QtGui import QImage

        self.w, self.h = w, h
        self.img = QImage(w, h, QImage.Format.Format_RGBA8888)
        self.img.fill(TRANSPARENT)

    def px(self, x, y, c):
        x, y = int(x), int(y)
        if 0 <= x < self.w and 0 <= y < self.h:
            self.img.setPixelColor(x, y, c)

    def rect(self, x, y, w, h, c):
        for yy in range(int(y), int(y + h)):
            for xx in range(int(x), int(x + w)):
                self.px(xx, yy, c)

    def frame(self, x, y, w, h, c):
        for xx in range(int(x), int(x + w)):
            self.px(xx, y, c)
            self.px(xx, y + h - 1, c)
        for yy in range(int(y), int(y + h)):
            self.px(x, yy, c)
            self.px(x + w - 1, yy, c)

    def hline(self, x, y, w, c):
        for xx in range(int(x), int(x + w)):
            self.px(xx, y, c)

    def vline(self, x, y, h, c):
        for yy in range(int(y), int(y + h)):
            self.px(x, yy, c)

    def text(self, text, x, y, scale=1, color=(255, 255, 255, 255), shadow=None):
        col = QColor(*color) if isinstance(color, tuple) else color
        if shadow is not None:
            scol = QColor(*shadow) if isinstance(shadow, tuple) else shadow
            pixelfont.draw_text(lambda px, py, _c: self.px(px, py, scol), text, x + scale, y + scale, scale)
        pixelfont.draw_text(lambda px, py, _c: self.px(px, py, col), text, x, y, scale)

    def text_outlined(self, text, x, y, scale, fill, outline, shadow=None):
        if shadow is not None:
            off = max(1, scale // 2)
            pixelfont.draw_text(
                lambda px, py, _c: self.rect(px + off, py + off, scale, scale, shadow),
                text, x, y, scale,
            )
        pixelfont.draw_text(
            lambda px, py, _c: self.rect(px - 1, py - 1, scale + 2, scale + 2, outline),
            text, x, y, scale,
        )
        pixelfont.draw_text(
            lambda px, py, _c: self.rect(px, py, scale, scale, fill),
            text, x, y, scale,
        )

    def blit(self, other, x, y):
        for yy in range(other.h):
            for xx in range(other.w):
                c = other.img.pixelColor(xx, yy)
                if c.alpha() > 0:
                    self.px(x + xx, y + yy, c)


# --------------------------------------------------------------------------
# Palettes
# --------------------------------------------------------------------------

P = dict(
    O=C("#080a10"),
    hair=C("#2a1a12"),
    hair_l=C("#4a2d1d"),
    skin=C("#e6b184"),
    shade=C("#c4815b"),
    shirt=C("#eef2f8"),
    shirt_d=C("#bcc4d2"),
    suit=C("#2b3c63"),
    suit_d=C("#19233d"),
    suit_l=C("#42598b"),
    tie=C("#cf3340"),
    tie_d=C("#7d1c27"),
    belt=C("#11151f"),
    shoe=C("#0d1018"),
    gun=C("#9aa5bb"),
    gun_d=C("#59637a"),
    gold=C("#e9b84a"),
)

E_RED = dict(
    P,
    suit=C("#6d2732"),
    suit_d=C("#3d131b"),
    suit_l=C("#93343f"),
    shirt=C("#d9d2c2"),
    shirt_d=C("#a49c8c"),
    hair=C("#17141a"),
    hair_l=C("#2c2730"),
    tie=C("#2b2f3a"),
    tie_d=C("#181b22"),
)

E_BLUE = dict(
    P,
    suit=C("#26505c"),
    suit_d=C("#122c35"),
    suit_l=C("#3a727f"),
    shirt=C("#d5d9d2"),
    shirt_d=C("#9fa59c"),
    hair=C("#101318"),
    hair_l=C("#242a33"),
    tie=C("#1f2530"),
    tie_d=C("#121720"),
)


def agent(ox, oy, pal, pose):
    """Draw a 16x24 agent in the given pose onto a fresh Pix."""
    a = Pix(16, 24)
    O = pal["O"]

    def px(x, y, c=O):
        a.px(x, y, c)

    # Legs first so the torso overlaps the hips cleanly.
    if pose == "dead":
        a.rect(2, 18, 9, 4, O)
        a.rect(3, 19, 8, 2, pal["suit"])
        a.rect(3, 21, 6, 1, pal["shoe"])
        a.rect(10, 17, 5, 5, O)
        a.rect(11, 18, 4, 3, pal["skin"])
        a.rect(11, 17, 4, 1, pal["hair"])
        a.px(14, 19, O)
        a.px(10, 21, pal["shirt"])
        return a

    if pose == "duck":
        # crouched: head at y=9, torso 14..20, folded legs
        a.rect(4, 9, 8, 7, O)
        a.rect(5, 10, 6, 5, pal["skin"])
        a.rect(5, 10, 6, 2, pal["hair"])
        a.rect(5, 11, 1, 2, pal["hair"])
        a.rect(10, 11, 1, 2, pal["hair"])
        a.px(7, 12, O)
        a.px(9, 12, O)
        a.rect(7, 14, 2, 1, pal["shade"])
        a.rect(3, 15, 10, 5, O)
        a.rect(4, 16, 8, 3, pal["suit"])
        a.rect(7, 15, 2, 3, pal["shirt"])
        a.px(8, 16, pal["tie"])
        a.px(8, 17, pal["tie"])
        a.rect(3, 15, 1, 4, pal["suit_d"])
        a.rect(12, 15, 1, 4, pal["suit_d"])
        a.rect(5, 20, 3, 2, O)
        a.rect(9, 20, 3, 2, O)
        a.rect(5, 20, 2, 1, pal["shoe"])
        a.rect(9, 20, 2, 1, pal["shoe"])
        a.rect(2, 18, 2, 2, pal["gun_d"])
        return a

    # Head
    a.rect(4, 2, 8, 7, O)
    a.rect(5, 3, 6, 6, pal["skin"])
    a.rect(5, 3, 6, 2, pal["hair"])
    a.rect(5, 3, 1, 2, pal["hair"])
    a.rect(10, 3, 1, 2, pal["hair"])
    # eyes / sunglasses
    if pal is E_RED or pal is E_BLUE:
        a.rect(6, 5, 4, 1, O)
        a.px(7, 5, pal["gun"])
        a.px(9, 5, pal["gun"])
        a.rect(5, 2, 6, 1, O)
        a.rect(5, 1, 6, 1, pal["hair"])
        a.rect(4, 2, 1, 1, pal["hair"])
        a.rect(11, 2, 1, 1, pal["hair"])
    else:
        a.px(7, 5, O)
        a.px(9, 5, O)
    a.rect(7, 7, 2, 1, pal["shade"])
    # neck
    a.rect(7, 8, 2, 2, pal["skin"])

    # Torso
    a.rect(4, 9, 8, 8, O)
    a.rect(5, 10, 6, 6, pal["suit"])
    a.rect(7, 9, 2, 5, pal["shirt"])
    a.rect(8, 10, 1, 4, pal["tie"])
    a.rect(6, 10, 1, 4, pal["suit_d"])
    a.rect(9, 10, 1, 4, pal["suit_d"])
    a.rect(4, 16, 8, 1, pal["belt"])

    # Arms
    a.rect(2, 10, 2, 6, O)
    a.rect(3, 11, 1, 4, pal["suit_d"])
    a.px(3, 15, pal["skin"])
    if pose == "shoot":
        a.rect(10, 11, 5, 4, O)
        a.rect(11, 12, 2, 2, pal["suit"])
        a.rect(13, 12, 2, 1, pal["gun"])
        a.px(13, 13, pal["gun_d"])
        a.px(15, 12, pal["gun"])
    else:
        a.rect(12, 10, 2, 6, O)
        a.rect(12, 11, 1, 4, pal["suit_d"])
        a.px(12, 15, pal["skin"])
        if pose == "ride":
            a.px(12, 14, pal["gold"])

    # Legs
    if pose in ("walk1", "walk2"):
        if pose == "walk1":
            # left foot planted back, right foot swinging forward
            a.rect(3, 17, 4, 5, O)
            a.rect(4, 17, 2, 4, pal["suit"])
            a.rect(2, 21, 4, 2, pal["shoe"])
            a.rect(9, 17, 4, 6, O)
            a.rect(10, 17, 2, 5, pal["suit"])
            a.rect(9, 22, 5, 2, pal["shoe"])
        else:
            # right foot lifted, left foot planted forward
            a.rect(4, 17, 4, 6, O)
            a.rect(5, 17, 2, 5, pal["suit"])
            a.rect(3, 22, 4, 2, pal["shoe"])
            a.rect(9, 17, 4, 5, O)
            a.rect(10, 17, 2, 4, pal["suit"])
            a.rect(10, 21, 4, 2, pal["shoe"])
    else:
        a.rect(4, 17, 4, 6, O)
        a.rect(5, 17, 2, 5, pal["suit"])
        a.rect(8, 17, 4, 6, O)
        a.rect(9, 17, 2, 5, pal["suit"])
        a.rect(4, 22, 4, 2, pal["shoe"])
        a.rect(9, 22, 3, 2, pal["shoe"])

    out = Pix(16, 24)
    out.blit(a, 0, 0)
    del ox, oy
    return out


def sheet(frames, cell=(16, 24)):
    s = Pix(cell[0] * len(frames), cell[1])
    for i, f in enumerate(frames):
        s.blit(f, i * cell[0], 0)
    return s


def door(kind, opened):
    d = Pix(13, 20)
    O = C("#07080c")
    d.rect(0, 0, 13, 20, O)
    if opened:
        d.rect(1, 1, 11, 18, C("#080b10"))
        glow = C("#f2c86a") if kind == "red" else C("#7fb8c9")
        d.rect(1, 1, 1, 18, glow)
        d.rect(1, 17, 11, 2, C("#241d16"))
        d.rect(2, 2, 10, 1, C("#171b22"))
        if kind == "red":
            # a document sitting in the lit opening
            d.rect(4, 9, 6, 7, C("#f4efe0"))
            d.rect(5, 10, 4, 1, C("#c9c0ac"))
            d.rect(5, 12, 4, 1, C("#c9c0ac"))
            d.rect(8, 14, 2, 2, C("#b32432"))
            d.px(4, 16, C("#d8d0bd"))
        else:
            d.rect(4, 3, 5, 10, C("#11151c"))
            d.rect(5, 4, 3, 8, C("#1b222d"))
        d.rect(11, 2, 1, 16, C("#2a2f3a"))
        return d

    if kind == "red":
        body = C("#a8202f")
        light = C("#cf3340")
        dark = C("#6d101c")
        d.rect(1, 1, 11, 18, body)
        d.rect(2, 2, 9, 16, light)
        d.rect(3, 5, 7, 10, body)
        d.rect(4, 6, 5, 8, dark)
        d.rect(3, 2, 7, 2, C("#ffe6a8"))
        d.rect(4, 3, 5, 1, C("#c9a35a"))
        d.rect(3, 16, 7, 2, dark)
        d.rect(9, 10, 1, 2, C("#e9b84a"))
        d.px(8, 10, C("#ffe6a8"))
        d.rect(4, 0, 5, 1, C("#3a1016"))
        d.rect(1, 0, 1, 1, C("#3a1016"))
        d.rect(11, 0, 1, 1, C("#3a1016"))
    else:
        body = C("#4b5361")
        light = C("#6b7484")
        dark = C("#2a303b")
        d.rect(1, 1, 11, 18, body)
        d.rect(2, 2, 9, 16, light)
        d.rect(3, 4, 7, 6, C("#161c26"))
        for yy in range(5, 10, 2):
            d.hline(3, yy, 7, C("#8e98aa"))
        d.rect(3, 12, 7, 5, dark)
        d.rect(9, 12, 1, 2, C("#b9c2d1"))
        for yy in (2, 6, 10, 14, 17):
            d.px(2, yy, C("#9aa3b2"))
            d.px(10, yy, C("#9aa3b2"))
    return d


def elevator(moving=False):
    e = Pix(22, 26)
    O = C("#07090e")
    frame = C("#7d879b")
    frame_d = C("#4a5264")
    frame_l = C("#aab3c4")
    e.rect(0, 0, 22, 26, O)
    e.rect(1, 1, 20, 24, C("#161b24"))
    # back panel
    e.rect(3, 4, 16, 17, C("#242c3a"))
    e.rect(4, 5, 14, 15, C("#1a2130"))
    # side pillars
    e.rect(1, 2, 2, 21, frame)
    e.rect(19, 2, 2, 21, frame)
    e.vline(1, 2, 21, frame_l)
    e.vline(20, 2, 21, frame_d)
    # top bar and lamp
    e.rect(2, 1, 18, 3, frame_d)
    e.rect(3, 2, 16, 1, frame_l)
    lamp = C("#ffb03a") if moving else C("#57e389")
    e.rect(10, 2, 2, 1, lamp)
    e.rect(9, 1, 4, 1, C("#0b0e14"))
    # floor plate
    e.rect(1, 21, 20, 4, frame_d)
    e.rect(2, 21, 18, 1, C("#c9a34a"))
    for x in range(2, 20, 4):
        e.rect(x, 22, 2, 2, C("#12151b"))
    # door frame detail
    e.vline(5, 4, 17, C("#3a4355"))
    e.vline(16, 4, 17, C("#3a4355"))
    e.rect(6, 5, 10, 1, C("#2e3746"))
    return e


def bullet(color):
    b = Pix(5, 2)
    b.rect(0, 0, 5, 2, color)
    b.rect(1, 0, 3, 1, C("#ffffff"))
    b.rect(4, 0, 1, 2, C("#ffffff"))
    return b


def doc_icon():
    d = Pix(7, 9)
    d.rect(1, 0, 5, 9, C("#f4efe0"))
    d.rect(1, 0, 5, 1, C("#ffffff"))
    d.rect(5, 0, 1, 2, C("#cfc6b2"))
    d.rect(2, 3, 3, 1, C("#b9b09d"))
    d.rect(2, 5, 3, 1, C("#b9b09d"))
    d.rect(2, 7, 2, 1, C("#b9b09d"))
    d.rect(4, 6, 2, 2, C("#b32432"))
    return d


def life_icon():
    p = Pix(8, 10)
    p.rect(1, 0, 6, 5, C("#e6b184"))
    p.rect(1, 0, 6, 2, C("#2a1a12"))
    p.px(3, 3, C("#080a10"))
    p.px(5, 3, C("#080a10"))
    p.rect(1, 5, 6, 5, C("#2b3c63"))
    p.rect(3, 5, 2, 3, C("#eef2f8"))
    return p


def exit_sign():
    e = Pix(40, 13)
    e.rect(0, 0, 40, 13, C("#07120b"))
    e.frame(1, 1, 38, 11, C("#1f7a3f"))
    e.text("EXIT", 4, 2, 1, color=(92, 255, 143, 255))
    # arrow
    e.rect(28, 6, 7, 1, C("#5cff8f"))
    e.px(34, 5, C("#5cff8f"))
    e.px(35, 6, C("#5cff8f"))
    e.px(34, 7, C("#5cff8f"))
    e.px(26, 5, C("#5cff8f"))
    e.px(26, 7, C("#5cff8f"))
    return e


def getaway_car():
    c = Pix(64, 24)
    O = C("#05070b")
    body = C("#22304d")
    body_l = C("#37507a")
    body_d = C("#121a2c")
    c.rect(1, 10, 62, 9, O)
    c.rect(2, 11, 60, 6, body)
    c.rect(2, 11, 60, 1, body_l)
    c.rect(8, 4, 40, 8, O)
    c.rect(10, 5, 36, 6, body)
    c.rect(12, 6, 13, 4, C("#7fb2cc"))
    c.rect(27, 6, 17, 4, C("#7fb2cc"))
    c.vline(26, 5, 6, body_d)
    c.rect(2, 15, 60, 2, body_d)
    # headlight + tail light
    c.rect(58, 12, 3, 2, C("#ffe6a8"))
    c.px(60, 13, C("#fff6d6"))
    c.rect(2, 12, 2, 2, C("#c8353f"))
    # wheels
    for wx in (12, 46):
        c.rect(wx, 17, 10, 7, O)
        c.rect(wx + 1, 18, 8, 5, C("#0c0f15"))
        c.rect(wx + 3, 19, 4, 3, C("#59637a"))
        c.px(wx + 4, 20, C("#9aa5bb"))
    # door seam and handle
    c.vline(33, 11, 5, body_d)
    c.rect(30, 12, 2, 1, C("#c9a34a"))
    # exhaust puff
    c.rect(0, 19, 2, 1, C("#5a6475", 160))
    return c


def logo():
    l = Pix(264, 96)
    # side arrows / elevator motif
    for x in (4, 256):
        l.rect(x, 8, 4, 52, C("#7d1420"))
        l.rect(x, 8, 2, 52, C("#d33340"))
    # down arrow left, up arrow right
    for i in range(5):
        l.rect(0 + i, 62 + i, 12 - 2 * i, 1, C("#d33340"))
        l.rect(258 + i, 50 - i, 12 - 2 * i, 1, C("#d33340"))
    l.text_outlined(
        "ELEVATOR", 38, 5, 4,
        fill=C("#f6e8c8"), outline=C("#0a0c12"), shadow=C("#8f1724"),
    )
    l.text_outlined(
        "ACTION", 62, 37, 4,
        fill=C("#e23a46"), outline=C("#0a0c12"), shadow=C("#4a0910"),
    )
    l.text("RED DOOR PROTOCOL", 82, 78, 1, color=C("#66e0d6"), shadow=C("#082b2b"))
    return l


# --------------------------------------------------------------------------
# Backgrounds
# --------------------------------------------------------------------------

THEMES = [
    dict(
        name="midnight",
        wall=C("#18223a"), wall2=C("#1d2946"), wall3=C("#111a2d"),
        trim=C("#33405e"), slab=C("#3d4660"), slab_l=C("#6f7b95"), slab_d=C("#242b3d"),
        accent=C("#54e0d2"), warm=C("#ffca6a"), shaft=C("#0a0e16"),
        rail=C("#4b5874"), rail_l=C("#7e8ca8"), street=C("#10141c"),
        neon=C("#ff5f8f"), poster=C("#2e4a72"),
    ),
    dict(
        name="amber",
        wall=C("#2c2119"), wall2=C("#35281e"), wall3=C("#201711"),
        trim=C("#5a4634"), slab=C("#5a4a3a"), slab_l=C("#8d7a5f"), slab_d=C("#33291e"),
        accent=C("#ff9d3c"), warm=C("#ffd27a"), shaft=C("#0d0906"),
        rail=C("#6e5942"), rail_l=C("#a38a68"), street=C("#151009"),
        neon=C("#ffcf4a"), poster=C("#5a3a24"),
    ),
    dict(
        name="violet",
        wall=C("#241a33"), wall2=C("#2e2140"), wall3=C("#191124"),
        trim=C("#4a3a62"), slab=C("#4b3d60"), slab_l=C("#7c6a95"), slab_d=C("#2b2238"),
        accent=C("#ff5fa2"), warm=C("#9fe8ff"), shaft=C("#09060e"),
        rail=C("#5c4a78"), rail_l=C("#8f78b0"), street=C("#0e0a14"),
        neon=C("#66e0d6"), poster=C("#3e2a5a"),
    ),
]

FLOORS_SCREEN = [30, 56, 82, 108, 134, 160, 186, 212]
SHAFT_X = [118, 220]
SHAFT_W = 22


def draw_bg(t):
    b = Pix(320, 240)
    rnd = random.Random(17)
    b.rect(0, 0, 320, 240, C("#05070b"))
    # outer wall + interior
    b.rect(8, 8, 304, 220, t["wall3"])
    b.rect(12, 10, 296, 216, t["wall"])
    b.rect(10, 8, 300, 2, t["trim"])

    # city peeking in thin strips at the very edges
    for side in (0, 314):
        for yy in range(12, 228, 6):
            if rnd.random() < 0.55:
                w = rnd.randint(4, 6)
                h = rnd.randint(2, 5)
                b.rect(side, yy, w, h, C("#0b1018"))
                for wx in range(side + 1, side + w - 1, 2):
                    if rnd.random() < 0.5:
                        b.px(wx, yy + 1, t["warm"])

    # wall bands for each floor
    prev = 10
    for f, sf in enumerate(FLOORS_SCREEN):
        top = prev + 1
        bottom = sf - 2
        b.rect(12, top, 296, max(0, bottom - top), t["wall"])
        # vertical wallpaper stripes
        for x in range(14, 306, 8):
            b.vline(x, top, max(0, bottom - top), t["wall2"])
        # chair rail / wainscot
        b.rect(12, bottom - 5, 296, 1, t["trim"])
        b.rect(12, bottom - 4, 296, 4, t["wall3"])
        # ceiling pipe
        b.hline(12, top + 1, 296, t["wall3"])
        b.hline(12, top + 2, 296, t["trim"])
        for x in range(20, 304, 48):
            b.rect(x, top + 3, 1, 1, t["rail_l"])
        # wall sconces + glow
        for x in range(24, 304, 56):
            b.rect(x, sf - 22, 3, 2, t["warm"])
            b.px(x + 1, sf - 23, C("#fff2c4"))
            b.rect(x - 1, sf - 21, 5, 1, QColor(t["warm"].red(), t["warm"].green(), t["warm"].blue(), 70))
            b.rect(x - 2, sf - 20, 7, 1, QColor(t["warm"].red(), t["warm"].green(), t["warm"].blue(), 35))
        # vent grilles near the ceiling
        for x in (64, 168, 272):
            b.rect(x, sf - 24, 9, 3, t["wall3"])
            for vx in range(x + 1, x + 8, 2):
                b.vline(vx, sf - 23, 1, t["trim"])
        # floor slab, broken by the shafts
        for x, w in slab_segments():
            b.rect(x, sf - 1, w, 3, t["slab"])
            b.hline(x, sf - 2, w, t["slab_l"])
            b.hline(x, sf + 1, w, t["slab_d"])
        prev = sf
        # floor number
        num = f"{7 - f:02d}" if f < 7 else "LL"
        b.text(num, 17, sf - 9, 1, color=QColor(t["accent"].red(), t["accent"].green(), t["accent"].blue(), 210))

    # bottom lobby floor
    b.rect(12, 212, 296, 15, C("#2a2a33"))
    for x in range(12, 308, 8):
        b.vline(x, 212, 15, C("#22222b"))
    b.hline(12, 212, 296, t["slab_l"])

    # street
    b.rect(8, 227, 304, 13, t["street"])
    b.hline(8, 227, 304, C("#3c4150"))
    for x in range(12, 308, 26):
        b.rect(x, 233, 12, 1, C("#4a4f5e"))

    # elevator shafts (drawn over the slabs, so they read as continuous)
    for sx in SHAFT_X:
        x0 = sx - SHAFT_W // 2
        b.rect(x0, 18, SHAFT_W, 200, t["shaft"])
        b.vline(x0, 18, 200, t["rail"])
        b.vline(x0 + 1, 18, 200, t["rail_l"])
        b.vline(x0 + SHAFT_W - 1, 18, 200, t["rail"])
        b.vline(x0 + SHAFT_W - 2, 18, 200, C("#1b2230"))
        # cable
        b.vline(sx, 18, 200, C("#59637a"))
        # cross braces
        for yy in range(24, 216, 13):
            b.hline(x0 + 2, yy, SHAFT_W - 4, C("#242c3a"))
            if yy % 26 == 24:
                b.rect(x0 + 3, yy, 1, 1, t["rail_l"])
                b.rect(x0 + SHAFT_W - 4, yy, 1, 1, t["rail_l"])
        # call buttons beside the opening, inside the shaft edge
        for sf in FLOORS_SCREEN:
            b.rect(x0 + 2, sf - 12, 3, 7, C("#0b0f16"))
            b.px(x0 + 3, sf - 10, t["accent"])
            b.px(x0 + 3, sf - 7, C("#5a6378"))
        # top machinery
        b.rect(x0 - 1, 18, SHAFT_W + 2, 3, C("#1c2330"))
        b.rect(x0 + 8, 14, 6, 4, C("#2a3344"))
        b.rect(x0 + 9, 13, 4, 1, t["rail_l"])

    # rooftop: a narrow parapet, leaving the top floor wall visible
    b.rect(8, 8, 304, 14, C("#161b25"))
    b.hline(8, 8, 304, C("#394154"))
    b.rect(8, 20, 304, 2, C("#2a3040"))
    b.hline(8, 20, 304, C("#5c6678"))
    b.hline(8, 21, 304, C("#111620"))
    for x in range(16, 300, 34):
        b.rect(x, 11, 10, 6, C("#202633"))
        b.hline(x, 11, 10, C("#394154"))
        b.px(x + 2, 13, C("#0d1016"))
        b.px(x + 6, 13, C("#0d1016"))
    # antenna
    b.rect(292, 2, 1, 20, C("#6b7486"))
    b.rect(288, 7, 9, 1, C("#6b7486"))
    b.rect(290, 11, 5, 1, C("#6b7486"))
    b.px(292, 1, C("#ff4d5a"))
    # neon hotel sign
    b.text("HOTEL", 228, 11, 1, color=t["neon"])
    b.rect(226, 19, 31, 1, QColor(t["neon"].red(), t["neon"].green(), t["neon"].blue(), 80))
    b.text("17", 296, 15, 1, color=t["warm"])
    b.text("07", 17, 22, 1, color=QColor(t["accent"].red(), t["accent"].green(), t["accent"].blue(), 210))

    # exit arch, ramp and sign at bottom right
    b.rect(262, 198, 44, 14, C("#0a0d12"))
    b.frame(262, 198, 44, 14, C("#3c4454"))
    # exit sign is a separate sprite, tinted by gameplay
    b.rect(268, 212, 30, 15, C("#070a0f"))
    b.rect(270, 213, 26, 12, C("#101720"))
    b.rect(270, 224, 26, 2, C("#1b2532"))
    b.rect(300, 226, 12, 2, C("#1b2532"))
    # lobby bench + plant
    b.rect(22, 205, 18, 2, C("#3a2a1c"))
    b.rect(23, 207, 2, 5, C("#2a1d12"))
    b.rect(37, 207, 2, 5, C("#2a1d12"))
    b.rect(50, 202, 3, 10, C("#2a3a24"))
    b.rect(47, 198, 9, 4, C("#375230"))
    b.rect(50, 209, 4, 3, C("#5a3a24"))
    b.text("LOBBY", 70, 203, 1, color=C("#8a93a6"))

    # wall grime / dither
    for _ in range(360):
        x = rnd.randint(13, 306)
        y = rnd.randint(11, 226)
        r = rnd.random()
        if r < 0.6:
            b.px(x, y, QColor(0, 0, 0, 16))
        else:
            b.px(x, y, QColor(255, 255, 255, 6))

    # scanlines + vignette
    for y in range(0, 240, 3):
        for x in range(0, 320):
            c = b.img.pixelColor(x, y)
            if c.alpha() > 0:
                b.px(x, y, QColor(int(c.red() * 0.93), int(c.green() * 0.93), int(c.blue() * 0.93), c.alpha()))

    for i in range(10):
        a = int(34 * (1 - i / 10))
        b.rect(i, 0, 1, 240, QColor(0, 0, 0, a))
        b.rect(319 - i, 0, 1, 240, QColor(0, 0, 0, a))
    for i in range(8):
        a = int(28 * (1 - i / 8))
        b.rect(0, i, 320, 1, QColor(0, 0, 0, a))
        b.rect(0, 239 - i, 320, 1, QColor(0, 0, 0, a))
    return b


def slab_segments():
    """Horizontal slab spans, with gaps for both elevator shafts."""
    spans = []
    x = 12
    for sx in SHAFT_X:
        left = sx - SHAFT_W // 2 - 1
        spans.append((x, left - x))
        x = sx + SHAFT_W // 2 + 1
    spans.append((x, 308 - x))
    return spans


def hide_background(doc):
    """Krita's createDocument adds an opaque white Background layer; hide it so
    the projection keeps real transparency."""
    for node in doc.rootNode().childNodes():
        if node.name() == "Background":
            node.setVisible(False)


def build_sprites():
    os.makedirs(SPRITES, exist_ok=True)
    board = Pix(320, 240)

    player_frames = [
        agent(0, 0, P, "idle"),
        agent(0, 0, P, "walk1"),
        agent(0, 0, P, "walk2"),
        agent(0, 0, P, "shoot"),
        agent(0, 0, P, "duck"),
        agent(0, 0, P, "ride"),
        agent(0, 0, P, "dead"),
    ]
    player_sheet = sheet(player_frames)
    board.blit(player_sheet, 0, 0)

    red_frames = [
        agent(0, 0, E_RED, "idle"),
        agent(0, 0, E_RED, "walk1"),
        agent(0, 0, E_RED, "walk2"),
        agent(0, 0, E_RED, "shoot"),
        agent(0, 0, E_RED, "dead"),
    ]
    red_sheet = sheet(red_frames)
    board.blit(red_sheet, 0, 28)

    blue_frames = [
        agent(0, 0, E_BLUE, "idle"),
        agent(0, 0, E_BLUE, "walk1"),
        agent(0, 0, E_BLUE, "walk2"),
        agent(0, 0, E_BLUE, "shoot"),
        agent(0, 0, E_BLUE, "dead"),
    ]
    blue_sheet = sheet(blue_frames)
    board.blit(blue_sheet, 0, 56)

    doors = [
        door("red", False),
        door("red", True),
        door("gray", False),
        door("gray", True),
    ]
    for i, d in enumerate(doors):
        board.blit(d, i * 13, 84)

    board.blit(elevator(False), 60, 84)
    board.blit(elevator(True), 86, 84)
    board.blit(getaway_car(), 114, 84)
    board.blit(bullet(C("#ffd447")), 0, 110)
    board.blit(bullet(C("#ff5a4a")), 8, 110)
    board.blit(doc_icon(), 16, 110)
    board.blit(life_icon(), 26, 110)
    board.blit(exit_sign(), 280, 84)
    board.blit(logo(), 0, 124)

    doc = ks.new_doc(320, 240, "sprite_board")
    hide_background(doc)
    lay = ks.new_layer(doc, "sprites")
    ks.write_pixels(lay, board.img)
    ks.save_kra(doc, os.path.join(ART, "sprite_board.kra"))
    doc.close()

    # export regions
    exports = {
        "player.png": (0, 0, 112, 24),
        "enemy_red.png": (0, 28, 80, 24),
        "enemy_blue.png": (0, 56, 80, 24),
        "door_red_closed.png": (0, 84, 13, 20),
        "door_red_open.png": (13, 84, 13, 20),
        "door_gray_closed.png": (26, 84, 13, 20),
        "door_gray_open.png": (39, 84, 13, 20),
        "elevator.png": (60, 84, 22, 26),
        "elevator_move.png": (86, 84, 22, 26),
        "car.png": (114, 84, 64, 24),
        "bullet_player.png": (0, 110, 5, 2),
        "bullet_enemy.png": (8, 110, 5, 2),
        "doc.png": (16, 110, 7, 9),
        "life.png": (26, 110, 8, 10),
        "exit_sign.png": (280, 84, 40, 13),
        "logo.png": (0, 124, 264, 96),
    }
    # re-open a doc for region exports (document was closed above)
    doc = ks.new_doc(320, 240, "sprite_board_export")
    hide_background(doc)
    lay = ks.new_layer(doc, "sprites")
    ks.write_pixels(lay, board.img)
    for name, (x, y, w, h) in exports.items():
        ks.export_region_png(doc, x, y, w, h, os.path.join(SPRITES, name))
    doc.close()
    ks.log(f"sprites: {len(exports)} PNGs")


def build_backgrounds():
    os.makedirs(BACKGROUNDS, exist_ok=True)
    strip = Pix(960, 240)
    for i, t in enumerate(THEMES):
        strip.blit(draw_bg(t), i * 320, 0)

    doc = ks.new_doc(960, 240, "backgrounds")
    hide_background(doc)
    lay = ks.new_layer(doc, "themes")
    ks.write_pixels(lay, strip.img)
    ks.save_kra(doc, os.path.join(ART, "backgrounds.kra"))
    for i, t in enumerate(THEMES):
        ks.export_region_png(doc, i * 320, 0, 320, 240, os.path.join(BACKGROUNDS, f"bg_{t['name']}.png"))
    doc.close()
    ks.log("backgrounds: 3 PNGs")


def __main__(args):
    try:
        os.makedirs(ART, exist_ok=True)
        build_sprites()
        build_backgrounds()
        ks.done()
    except BaseException:
        traceback.print_exc(file=sys.stderr)
        raise
