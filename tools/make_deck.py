#!/usr/bin/env python3
"""Build the Elevator Action project deck with LibreOffice Impress + UNO."""

from pathlib import Path

import uno
from com.sun.star.beans import PropertyValue
from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "deck"
OUT.mkdir(exist_ok=True)

W, H = 33867, 19050  # 13.333 x 7.5 in in 1/100 mm

BG = 0x080C14
PANEL = 0x0D1624
PANEL2 = 0x111C2E
CY = 0x5EE0D5
RED = 0xD93442
CREAM = 0xF4E7C6
WHITE = 0xEEF2F8
GREY = 0x93A1B7
DIM = 0x5E6B80
GOLD = 0xFFD463
GREEN = 0x5CFF8F
DARKRED = 0x2A0D13


def pv(name, value):
    p = PropertyValue()
    p.Name = name
    p.Value = value
    return p


def enum(type_name, value):
    return uno.Enum(type_name, value)


def connect():
    local = uno.getComponentContext()
    resolver = local.ServiceManager.createInstanceWithContext(
        "com.sun.star.bridge.UnoUrlResolver", local
    )
    ctx = resolver.resolve(
        "uno:socket,host=127.0.0.1,port=2002;urp;StarOffice.ComponentContext"
    )
    smgr = ctx.ServiceManager
    desktop = smgr.createInstanceWithContext("com.sun.star.frame.Desktop", ctx)
    return ctx, smgr, desktop


CTX, SMGR, DESKTOP = connect()
DOC = None
HIDDEN = (pv("Hidden", True),)


def new_slide(doc, index):
    pages = doc.DrawPages
    if index == 0:
        page = pages.getByIndex(0)
    else:
        pages.insertNewByIndex(index)
        page = pages.getByIndex(index)
    page.Width = W
    page.Height = H
    page.Layout = 0
    while page.Count:
        page.remove(page.getByIndex(0))
    add_rect(page, 0, 0, W, H, BG)
    return page


def add_rect(slide, x, y, w, h, fill, line=None, line_w=30, radius=0, transp=0, name=None):
    shape = DOC.createInstance("com.sun.star.drawing.RectangleShape")
    pt = uno.createUnoStruct("com.sun.star.awt.Point")
    pt.X, pt.Y = int(x), int(y)
    shape.Position = pt
    sz = uno.createUnoStruct("com.sun.star.awt.Size")
    sz.Width, sz.Height = int(w), int(h)
    shape.Size = sz
    shape.FillStyle = enum("com.sun.star.drawing.FillStyle", "SOLID")
    shape.FillColor = fill
    shape.FillTransparence = int(transp)
    if line is None:
        shape.LineStyle = enum("com.sun.star.drawing.LineStyle", "NONE")
    else:
        shape.LineStyle = enum("com.sun.star.drawing.LineStyle", "SOLID")
        shape.LineColor = line
        shape.LineWidth = line_w
    if radius:
        try:
            shape.CornerRadius = radius
        except Exception:
            pass
    if name:
        shape.Name = name
    slide.add(shape)
    return shape


def add_line(slide, x1, y1, x2, y2, color, width=40):
    shape = DOC.createInstance("com.sun.star.drawing.LineShape")
    pt = uno.createUnoStruct("com.sun.star.awt.Point")
    pt.X, pt.Y = int(min(x1, x2)), int(min(y1, y2))
    shape.Position = pt
    sz = uno.createUnoStruct("com.sun.star.awt.Size")
    sz.Width, sz.Height = int(abs(x2 - x1)), int(abs(y2 - y1))
    shape.Size = sz
    shape.LineColor = color
    shape.LineWidth = width
    slide.add(shape)
    return shape


def add_text(
    slide,
    text,
    x,
    y,
    w,
    h,
    size=16,
    color=WHITE,
    font="DejaVu Sans",
    bold=False,
    align="LEFT",
    valign="TOP",
    margin=0,
    name=None,
    spacing=None,
):
    shape = DOC.createInstance("com.sun.star.drawing.TextShape")
    pt = uno.createUnoStruct("com.sun.star.awt.Point")
    pt.X, pt.Y = int(x), int(y)
    shape.Position = pt
    sz = uno.createUnoStruct("com.sun.star.awt.Size")
    sz.Width, sz.Height = int(w), int(h)
    shape.Size = sz
    shape.TextAutoGrowHeight = False
    shape.TextWordWrap = True
    shape.TextLeftDistance = shape.TextRightDistance = shape.TextUpperDistance = shape.TextLowerDistance = int(margin)
    shape.TextVerticalAdjust = enum("com.sun.star.drawing.TextVerticalAdjust", valign)
    shape.FillStyle = enum("com.sun.star.drawing.FillStyle", "NONE")
    shape.LineStyle = enum("com.sun.star.drawing.LineStyle", "NONE")
    slide.add(shape)
    shape.Text.setString(text)
    cur = shape.Text.createTextCursor()
    cur.gotoEnd(True)
    cur.CharFontName = font
    cur.CharHeight = size
    cur.CharColor = color
    cur.CharWeight = 150.0 if bold else 100.0
    cur.ParaAdjust = enum("com.sun.star.style.ParagraphAdjust", align)
    if spacing is not None:
        ls = uno.createUnoStruct("com.sun.star.style.LineSpacing")
        ls.Mode = 0
        ls.Height = int(spacing)
        cur.ParaLineSpacing = ls
    if name:
        shape.Name = name
    return shape


def add_image(slide, path, x, y, w, h, border=None, border_w=45, name=None):
    path = Path(path)
    if border is not None:
        add_rect(slide, x - border_w, y - border_w, w + border_w * 2, h + border_w * 2, 0x05080D, border, border_w)
    shape = DOC.createInstance("com.sun.star.drawing.GraphicObjectShape")
    pt = uno.createUnoStruct("com.sun.star.awt.Point")
    pt.X, pt.Y = int(x), int(y)
    shape.Position = pt
    sz = uno.createUnoStruct("com.sun.star.awt.Size")
    sz.Width, sz.Height = int(w), int(h)
    shape.Size = sz
    provider = SMGR.createInstanceWithContext("com.sun.star.graphic.GraphicProvider", CTX)
    shape.Graphic = provider.queryGraphic((pv("URL", uno.systemPathToFileUrl(str(path.resolve()))),))
    if name:
        shape.Name = name
    slide.add(shape)
    return shape


def add_image_fit(slide, path, x, y, max_w, max_h, border=0x283A57, border_w=40):
    iw, ih = Image.open(path).size
    scale = min(max_w / iw, max_h / ih)
    w, h = iw * scale, ih * scale
    ix = x + (max_w - w) / 2
    iy = y + (max_h - h) / 2
    add_image(slide, path, ix, iy, w, h, border, border_w)
    return ix, iy, w, h


def chrome(slide, idx, total, kicker="RED DOOR PROTOCOL // BEVY 0.20.1"):
    add_text(slide, kicker, 1000, 560, 17000, 420, size=10, color=CY, font="Arcade 17")
    add_text(slide, "ELEVATOR ACTION", 22000, 560, 10800, 420, size=10, color=DIM, font="Arcade 17", align="RIGHT")
    add_text(slide, f"{idx:02d} / {total:02d}", 30000, 18000, 2600, 500, size=10, color=DIM, font="Arcade 17", align="RIGHT")
    add_rect(slide, 1000, 1010, 900, 45, RED, name="kicker-rule")


def heading(slide, title, kicker=None):
    y = 1050
    if kicker:
        add_text(slide, kicker, 1000, y, 31000, 500, size=12, color=CY, font="Arcade 17")
        y += 650
    add_text(slide, title, 1000, y, 31000, 1300, size=32, color=CREAM, font="Arcade 17")
    add_rect(slide, 1000, y + 1200, 1500, 50, RED)


def chip(slide, x, y, w, h, label, value, color=CY):
    add_rect(slide, x, y, w, h, PANEL2, 0x23344F, 30)
    add_text(slide, value, x + 240, y + 150, w - 480, 900, size=30, color=color, font="Arcade 17")
    add_text(slide, label, x + 240, y + 1080, w - 480, 420, size=10, color=GREY, font="Arcade 17")


def new_doc():
    doc = DESKTOP.loadComponentFromURL("private:factory/simpress", "_blank", 0, HIDDEN)
    return doc


def build():
    global DOC
    doc = new_doc()
    DOC = doc
    total = 10

    # ---------------------------------------------------------------- 1 title
    s = new_slide(doc, 0)
    iw, ih = Image.open(ROOT / "assets/art/banner.png").size
    bw = W
    bh = bw * ih / iw
    add_image(s, ROOT / "assets/art/banner.png", 0, (H - bh) / 2, bw, bh, border=None)
    add_text(
        s,
        "PLAYABLE BUILD  /  RUST + BEVY 0.20.1  /  2026",
        1200,
        H - 900,
        22000,
        420,
        size=10,
        color=GREY,
        font="Arcade 17",
    )
    add_text(s, "01 / 10", 30000, H - 900, 2600, 420, size=10, color=DIM, font="Arcade 17", align="RIGHT")

    # ---------------------------------------------------------------- 2 brief
    s = new_slide(doc, 1)
    chrome(s, 2, total)
    heading(s, "A COMPLETE ARCADE GAME", "THE BRIEF")
    add_text(
        s,
        "Rebuild the 1983 elevator-spy loop as a complete,\nplayable 2D arcade game — not a tech demo.",
        1000,
        3300,
        15000,
        1400,
        size=20,
        color=WHITE,
    )
    add_text(
        s,
        "Descend a cutaway hotel. Call and ride two elevators.\nCrack the red document doors. Dodge the night desk's\nagents. Collect every file, then reach the getaway car.",
        1000,
        4800,
        15300,
        2200,
        size=17,
        color=GREY,
        spacing=115,
    )
    chip(s, 1000, 7600, 4800, 1800, "STAGES", "3", RED)
    chip(s, 6100, 7600, 4800, 1800, "FLOORS", "8", CY)
    chip(s, 11200, 7600, 4800, 1800, "ELEVATORS", "2", GOLD)
    add_text(
        s,
        "One screen. Three escalating assignments.\nNo save points, no continues — just three lives.",
        1000,
        10100,
        15000,
        1300,
        size=15,
        color=DIM,
    )
    add_text(
        s,
        "ORIGINAL ART  /  ORIGINAL AUDIO  /  NO ARCADE ROMS",
        1000,
        H - 1500,
        16000,
        420,
        size=10,
        color=CY,
        font="Arcade 17",
    )
    add_image_fit(s, ROOT / "shots/stage1.png", 17200, 2100, 15400, 12500, border=RED, border_w=50)
    add_text(s, "STAGE 1 — THE HOTEL MERIDIAN", 17200, 14900, 15400, 450, size=11, color=GREY, font="Arcade 17")

    # ---------------------------------------------------------------- 3 loop
    s = new_slide(doc, 2)
    chrome(s, 3, total)
    heading(s, "SIX FILES. NO EXIT.", "THE LOOP")
    steps = [
        ("01", "CALL THE CAR", "W at a shaft summons an elevator. W again boards it once it docks."),
        ("02", "RIDE THE FLOORS", "W/S picks the next stop. A/D steps off when the car is docked."),
        ("03", "CRACK THE RED DOORS", "Walk into a red door to take its file: +500. Collect them all to open the exit."),
        ("04", "REACH THE CAR", "Walk into the getaway car at the bottom right. Bonus scales with lives."),
    ]
    y = 3150
    for num, title, body in steps:
        add_text(s, num, 1000, y, 3000, 1100, size=24, color=RED, font="Arcade 17", name=f"step-{num}")
        add_text(s, title, 1000, y + 800, 14800, 600, size=17, color=CREAM, font="Arcade 17")
        add_text(s, body, 1000, y + 1450, 14800, 950, size=14, color=GREY)
        add_line(s, 1000, y + 2550, 16200, y + 2550, 0x1E2C43, 25)
        y += 3000
    add_image_fit(s, ROOT / "shots/stage3.png", 17700, 2100, 15000, 12000, border=RED, border_w=50)
    add_text(s, "STAGE 3 — THE RED DOOR PROTOCOL", 17700, 14400, 15000, 450, size=11, color=GREY, font="Arcade 17")
    add_rect(s, 1000, 15900, 31700, 1000, DARKRED, RED, 35)
    add_text(
        s,
        "RED = FILES.   GRAY = AGENTS.   THE CAR OPENS ONLY WHEN EVERY FILE IS YOURS.",
        1500,
        16130,
        30700,
        500,
        size=13,
        color=CREAM,
        font="Arcade 17",
    )

    # ---------------------------------------------------------------- 4 feel
    s = new_slide(doc, 3)
    chrome(s, 4, total)
    heading(s, "DUCK. JUMP. SHOOT.", "FEEL")
    controls = [
        ("MOVE", "A / D  or  ← / →"),
        ("SHOOT", "J / X  (hold to autofire)"),
        ("JUMP", "K / Space"),
        ("DUCK", "S / ↓  (dodges chest shots)"),
        ("ELEVATOR", "W / ↑ call, board, next floor"),
        ("PAUSE / MUTE", "P or Esc   /   M"),
    ]
    y = 3300
    for label, keys in controls:
        add_text(s, label, 1000, y, 4200, 500, size=11, color=CY, font="Arcade 17")
        add_text(s, keys, 5200, y - 40, 12000, 500, size=16, color=CREAM)
        add_line(s, 1000, y + 620, 15400, y + 620, 0x1E2C43, 25)
        y += 920
    add_text(s, "THE FEEL", 1000, 9700, 5000, 450, size=12, color=RED, font="Arcade 17")
    add_text(
        s,
        "One-hit deaths with 2.4 s of spawn protection keep\n"
        "mistakes legible. Ducking swaps to a 10 px hitbox\n"
        "at the feet, so chest-height bullets pass overhead.\n"
        "Agents telegraph for 0.55–0.9 s before firing.",
        1000,
        10200,
        14200,
        2200,
        size=15,
        color=GREY,
        spacing=115,
    )
    add_image_fit(s, ROOT / "assets/art/action_crop.png", 17000, 2200, 16000, 12000, border=RED, border_w=50)
    add_text(s, "MUZZLE FLASH, TWO ROUNDS, AN OPEN RED DOOR", 17000, 14400, 16000, 450, size=11, color=GREY, font="Arcade 17")

    # ---------------------------------------------------------------- 5 architecture
    s = new_slide(doc, 4)
    chrome(s, 5, total)
    heading(s, "BEVY ECS, TOP TO BOTTOM", "UNDER THE HOOD")
    boxes = [
        (1000, 3100, "STATES", "Title · InGame\nGameOver · Victory", RED),
        (8800, 3100, "PLAYER", "move · jump · duck\nshoot · ride · respawn", CY),
        (16600, 3100, "ENEMY", "floor-aware chase\ncall · board · shoot", CY),
        (24400, 3100, "ELEVATOR", "CarState resource\nposition · target · riders", GOLD),
        (1000, 6200, "COMBAT", "bullets · particles\nhits · doors · exit", CY),
        (8800, 6200, "LEVEL", "spawn · waves\ntransitions", CY),
        (16600, 6200, "UI", "HUD · banners\npause · screens", CY),
        (24400, 6200, "WORLD", "layout · levels\npure math", GOLD),
    ]
    for x, y, name, desc, col in boxes:
        add_rect(s, x, y, 7000, 1900, PANEL2, 0x23344F, 30)
        add_rect(s, x, y, 90, 1900, col)
        add_text(s, name, x + 320, y + 180, 6400, 500, size=17, color=col, font="Arcade 17")
        add_text(s, desc, x + 320, y + 800, 6400, 900, size=14, color=GREY, spacing=110)
    add_text(
        s,
        f"{'COMPONENTS':<12}Player · Enemy · Door · Bullet · ElevatorCar\n"
        f"{'RESOURCES':<12}RunState · Elevators · Fx · PendingLevel\n"
        f"{'MESSAGES':<12}PlaySfx\n"
        f"{'SCHEDULES':<12}Startup · Update · OnEnter(state)",
        1000,
        9000,
        16000,
        2600,
        size=13,
        color=GREY,
        font="Liberation Mono",
        spacing=125,
    )
    add_rect(s, 1000, 12100, 31700, 1200, PANEL, 0x23344F, 30)
    add_text(
        s,
        "ONE PLUGIN  ·  8 MODULES  ·  NO unsafe  ·  NO CUSTOM RENDERER",
        1500,
        12360,
        30700,
        500,
        size=13,
        color=CREAM,
        font="Arcade 17",
    )
    add_text(
        s,
        "Systems are chained in a single Update schedule, gated by state\n"
        "and by a Normal-phase run condition. Commands, messages and\n"
        "despawn-on-exit markers keep level teardown and restarts clean.",
        1000,
        14000,
        31700,
        2200,
        size=15,
        color=GREY,
        spacing=115,
    )

    # ---------------------------------------------------------------- 6 elevator
    s = new_slide(doc, 5)
    chrome(s, 6, total)
    heading(s, "THE ELEVATOR IS THE LEVEL", "SIMULATION")
    add_rect(s, 1000, 3100, 15700, 8800, 0x0B1220, 0x23344F, 30)
    code = (
        "pub struct CarState {\n"
        "    pub bottom: f32,\n"
        "    pub target_floor: usize,\n"
        "    pub moving: bool,\n"
        "    pub riders: u32,\n"
        "}\n"
        "fn update(&mut self, dt: f32) -> Option<usize> {\n"
        "    let target = car_bottom_for_floor(self.target_floor);\n"
        "    let delta = target - self.bottom;\n"
        "    let step = CAR_SPEED * dt;\n"
        "    if delta.abs() <= step {\n"
        "        self.bottom = target;     // dock\n"
        "        self.moving = false;\n"
        "        Some(self.target_floor)\n"
        "    } else {\n"
        "        self.bottom += delta.signum() * step;\n"
        "        None\n"
        "    }\n"
        "}"
    )
    add_text(s, code, 1400, 3400, 15000, 8100, size=11, color=0xC8E6E1, font="Liberation Mono", spacing=100)
    add_text(s, "src/world.rs", 1400, 12000, 6000, 400, size=10, color=DIM, font="Arcade 17")
    add_text(s, "WHAT MAKES IT WORK", 17700, 3200, 15000, 500, size=12, color=RED, font="Arcade 17")
    notes = [
        "request() retargets a moving car, so a rider can change floors mid-trip.",
        "44 px/s and a 26 px floor pitch: the ride reads as deliberate, not floaty.",
        "docked_floor() is the single truth for boarding, stepping off, and enemy AI.",
        "A riders counter stops agents from calling a car out from under the player.",
        "Agents walk to a shaft, call, board, retarget, and dismount — the same rules as the player.",
    ]
    y = 3900
    for n in notes:
        add_text(s, "▸", 17700, y, 500, 500, size=15, color=RED, font="DejaVu Sans")
        add_text(s, n, 18400, y, 14200, 1200, size=15, color=GREY, spacing=110)
        y += 1550
    add_rect(s, 17700, 12300, 15800, 800, DARKRED, RED, 30)
    add_text(s, "TESTS  car_rides_and_docks  ·  car_does_not_move_when_already_there", 18100, 12450, 15200, 400, size=9, color=CREAM, font="Arcade 17")
    add_image_fit(s, ROOT / "assets/sprites/elevator.png", 26700, 14200, 2400, 2700, border=0x23344F, border_w=30)
    add_image_fit(s, ROOT / "assets/sprites/elevator_move.png", 29500, 14200, 2400, 2700, border=0x23344F, border_w=30)
    add_text(s, "IDLE", 26500, 17000, 2800, 400, size=9, color=GREY, font="Arcade 17", align="CENTER")
    add_text(s, "MOVING", 29300, 17000, 2800, 400, size=9, color=GREY, font="Arcade 17", align="CENTER")

    # ---------------------------------------------------------------- 7 art
    s = new_slide(doc, 6)
    chrome(s, 7, total)
    heading(s, "PIXELS FROM SCRATCH", "ART PIPELINE")
    add_image_fit(s, ROOT / "assets/art/sprite_contact.png", 1000, 3100, 17500, 9700, border=0x23344F, border_w=35)
    add_text(s, "1× SOURCE ART — 7 PLAYER FRAMES, 2 AGENT SHEETS, DOORS, CARS, HUD", 1000, 13000, 17500, 450, size=10, color=GREY, font="Arcade 17")
    notes = [
        ("KRITA 6 / PYKRITA", "Every sprite and all three backdrops are drawn at 1× into QImages, pushed through Krita layers, and exported as PNG. Editable .kra sources ship in assets/art/."),
        ("A 5×7 FONT, TYPESET IN CODE", "A hand-built glyph table is expanded to a real TTF with fontTools, then loaded by Bevy for crisp HUD and menu text."),
        ("INKSCAPE", "The README banner and social preview are vector SVG, rendered to a 1280×640 PNG."),
        ("NO EXTERNAL ART", "No arcade rips, no stock sprites, no sample packs. Backgrounds are midnight, amber, and violet themes built from the same floor plan."),
    ]
    y = 3100
    for title, body in notes:
        add_text(s, title, 19500, y, 13500, 500, size=12, color=CY, font="Arcade 17")
        add_text(s, body, 19500, y + 600, 13500, 1700, size=14, color=GREY, spacing=110)
        y += 2850
    add_image_fit(s, ROOT / "assets/art/themes.png", 19500, 13900, 13500, 3500, border=0x23344F, border_w=30)
    add_text(s, "THREE BUILDINGS, ONE FLOOR PLAN", 19500, 17600, 13500, 400, size=9, color=DIM, font="Arcade 17")

    # ---------------------------------------------------------------- 8 sound/presentation
    s = new_slide(doc, 7)
    chrome(s, 8, total)
    heading(s, "A 16-SECOND SPY LOOP", "SOUND + PRESENTATION")
    add_image_fit(s, ROOT / "assets/art/waveform.png", 1000, 3100, 18500, 4400, border=0x23344F, border_w=35)
    add_text(s, "ORIGINAL CHIPTUNE — SQUARE, TRIANGLE AND NOISE, RENDERED TO WAV", 1000, 7700, 18500, 450, size=10, color=GREY, font="Arcade 17")
    bullets = [
        "11 cues plus the loop: pistol, enemy shot, hit, pickup, death, ding, door, call, clear, game over, victory.",
        "22.05 kHz mono; all synthesized in pure Python with the standard library.",
        "One GlobalVolume resource handles mute without touching individual sinks.",
    ]
    y = 8500
    for b in bullets:
        add_text(s, "▸", 1000, y, 400, 450, size=15, color=RED)
        add_text(s, b, 1600, y, 17000, 1200, size=15, color=GREY, spacing=110)
        y += 1550
    add_image_fit(s, ROOT / "assets/art/hud_crop.png", 1000, 13500, 18500, 2900, border=0x23344F, border_w=30)
    add_text(s, "CRISP HUD AT 3× — CUSTOM FONT, NO ANTIALIASING", 1000, 16700, 18500, 400, size=9, color=DIM, font="Arcade 17")
    add_rect(s, 20400, 3100, 12500, 3000, PANEL2, 0x23344F, 30)
    add_text(s, "320 × 240", 20800, 3320, 11700, 1100, size=36, color=CREAM, font="Arcade 17")
    add_text(s, "VIRTUAL ARCADE SCREEN", 20800, 4520, 11700, 400, size=10, color=CY, font="Arcade 17")
    add_text(s, "rendered at 3× in a fixed 960×720 window", 20800, 5100, 11700, 500, size=13, color=GREY)
    add_rect(s, 20400, 6400, 12500, 3000, PANEL2, 0x23344F, 30)
    add_text(s, "NEAREST", 20800, 6620, 11700, 1100, size=36, color=CREAM, font="Arcade 17")
    add_text(s, "SAMPLING + INTEGER SNAP", 20800, 7820, 11700, 400, size=10, color=CY, font="Arcade 17")
    add_text(s, "no blurry pixels, ever", 20800, 8400, 11700, 500, size=13, color=GREY)
    add_image_fit(s, ROOT / "shots/pause.png", 20400, 9800, 12500, 8000, border=RED, border_w=40)

    # ---------------------------------------------------------------- 9 verification
    s = new_slide(doc, 8)
    chrome(s, 9, total)
    heading(s, "TRUST, BUT VERIFY", "VERIFICATION")
    imgs = [
        ("shots/title.png", "TITLE"),
        ("shots/stage1.png", "STAGE 1"),
        ("shots/stage3.png", "STAGE 3"),
        ("shots/pause.png", "PAUSE"),
    ]
    x = 1000
    for path, label in imgs:
        add_image_fit(s, ROOT / path, x, 3100, 7200, 5400, border=0x23344F, border_w=35)
        add_text(s, label, x, 8600, 7200, 400, size=10, color=GREY, font="Arcade 17", align="CENTER")
        x += 7900
    add_text(s, "EVERY IMAGE IN THIS DECK WAS CAPTURED FROM THE RUNNING BINARY.", 1000, 9600, 31700, 500, size=12, color=CY, font="Arcade 17")
    checks = [
        ("FMT", "cargo fmt --check"),
        ("LINT", "clippy -D warnings"),
        ("TESTS", "15 passing"),
        ("CAPTURE", "self-screenshot"),
        ("DEMO", "scripted input"),
    ]
    x = 1000
    for i, (label, value) in enumerate(checks):
        add_rect(s, x, 10700, 5800, 1500, PANEL2, 0x23344F, 30)
        add_text(s, label, x + 250, 10900, 5300, 400, size=10, color=RED, font="Arcade 17")
        add_text(s, value, x + 250, 11400, 5300, 500, size=13, color=CREAM, font="DejaVu Sans Mono")
        x += 6200
    add_text(
        s,
        "Unit tests cover pure elevator/collision math and every level layout. Integration tests\n"
        "drive full ECS apps for document pickup, damage/life loss, respawn, stage advance,\n"
        "victory transition, and the complete level-spawn pipeline. Screenshot mode waits for\n"
        "the scene to settle, saves the framebuffer, and exits — the loop that caught UI\n"
        "overlaps, font fallback squares, and HUD/playfield collisions.",
        1000,
        13000,
        31700,
        3500,
        size=15,
        color=GREY,
        spacing=120,
    )

    # ---------------------------------------------------------------- 10 play
    s = new_slide(doc, 9)
    chrome(s, 10, total)
    heading(s, "PLAY IT", "SHIP")
    add_text(
        s,
        "A complete, original arcade game in Rust.\n"
        "Clone it, build it, crack some red doors.",
        1000,
        3200,
        20000,
        1400,
        size=20,
        color=GREY,
        spacing=120,
    )
    add_rect(s, 1000, 5100, 20500, 3900, 0x0B1220, 0x23344F, 30)
    add_text(
        s,
        "git clone https://github.com/orbitmechanic/elevator-action-bevy\n"
        "cd elevator-action-bevy\n"
        "cargo run\n"
        "cargo test\n"
        "cargo clippy --all-targets -- -D warnings",
        1500,
        5450,
        19500,
        3200,
        size=13,
        color=0xC8E6E1,
        font="Liberation Mono",
        spacing=125,
    )
    add_text(s, "MIT LICENSE  ·  ORIGINAL ART + AUDIO  ·  BEVY 0.20.1", 1000, 9700, 20500, 450, size=12, color=CY, font="Arcade 17")
    add_text(
        s,
        "Built with Krita, Inkscape, LibreOffice and a lot of\npixel counting. Thanks for playing.",
        1000,
        12000,
        20000,
        1400,
        size=16,
        color=GREY,
        spacing=120,
    )
    add_rect(s, 23200, 3600, 7900, 7900, 0xEEF2F8, None)
    add_image(s, ROOT / "assets/art/repo_qr.png", 24500, 4900, 5300, 5300, border=None)
    add_text(s, "SCAN TO PLAY", 23200, 11700, 7900, 500, size=13, color=CREAM, font="Arcade 17", align="CENTER")
    add_text(s, "github.com/orbitmechanic", 23200, 12700, 7900, 400, size=9, color=GREY, font="Arcade 17", align="CENTER")
    add_rect(s, 1000, 15900, 31700, 900, DARKRED, RED, 30)
    add_text(
        s,
        "PRESS ENTER TO START",
        1500,
        16140,
        30700,
        450,
        size=13,
        color=CREAM,
        font="Arcade 17",
        align="CENTER",
    )

    doc.DrawPages.getByIndex(0).Name = "Title"
    for i in range(1, total):
        doc.DrawPages.getByIndex(i).Name = f"Slide {i+1}"

    odp = OUT / "elevator_action_deck.odp"
    pdf = OUT / "elevator_action_deck.pdf"
    doc.storeToURL(uno.systemPathToFileUrl(str(odp)), (pv("FilterName", "impress8"),))
    doc.storeToURL(uno.systemPathToFileUrl(str(pdf)), (pv("FilterName", "impress_pdf_Export"),))
    doc.close(False)
    print(f"wrote {odp}")
    print(f"wrote {pdf}")


if __name__ == "__main__":
    build()
