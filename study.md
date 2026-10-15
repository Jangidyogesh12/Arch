# Line Draw Tool on a Canvas — Study Notes

This document is a self-contained study guide for building a **fixed-size line drawing
tool**. It is split into two halves:

1. **Part A — the math.** What a canvas really is, how a line is defined
   mathematically, and how we turn a continuous line into discrete pixels
   (rasterization). This is the "computer graphics" theory.
2. **Part B — how the code will work.** How to map that math onto *this* repo's
   architecture: the Rust `engine`, the `wasm-bindgen` `wrapper`, and the TypeScript
   `frontend`.

No implementation code is given — only the reasoning, formulas, and a checklist of
what to build.

---

# PART A — THE MATH

## 1. What a canvas actually is

### 1.1 A canvas is a grid of pixels

An HTML `<canvas>` of size `W × H` is, at the lowest level, a **2D array of
colored squares (pixels)**. There are exactly `W * H` pixels. Each pixel has a
position `(x, y)` where:

- `x` is the **column** index, `0` on the left, `W - 1` on the right.
- `y` is the **row** index, `0` at the **top**, `H - 1` at the **bottom**.

The important, easily-forgotten fact: **on a canvas, the y-axis points downward.**
This is opposite to the math convention (y-axis points up). Every coordinate
you get from a mouse event already lives in this "y-down" world, so for line
drawing you usually do *not* flip anything — but be aware of it the moment you
start mixing in "math-style" coordinates.

### 1.2 The framebuffer and its memory layout

The raw data behind a canvas is called the **framebuffer**: a flat block of bytes,
one 4-byte group per pixel, in **RGBA** order (Red, Green, Blue, Alpha), each
channel 0–255. This is the same layout the browser hands you via `ImageData`.

Because the framebuffer is flat (1-dimensional) but the image is 2D, we need a
formula to find the byte offset of pixel `(x, y)`. This is **row-major** order:
all of row 0 first, then row 1, etc.

```
byteOffset(x, y) = (y * W + x) * 4
```

Then:

- `byteOffset + 0` = Red
- `byteOffset + 1` = Green
- `byteOffset + 2` = Blue
- `byteOffset + 3` = Alpha

This single formula is the entire "address system" of the image. `set_pixel` is
just writing 4 bytes here. `get_pixel` is reading them back. Everything else in
this document eventually calls one of those two operations.

### 1.3 Coordinate systems you will juggle

There are three coordinate spaces to keep straight:

| Space | Origin | y direction | Units |
|-------|--------|-------------|-------|
| **Client / page** | top-left of the viewport | down | CSS pixels |
| **Canvas / device** | top-left of the canvas element | down | framebuffer pixels |
| **Math / NDC** (optional) | center of the canvas | up | normalized `[-1, 1]` |

You receive events in **client** space, you draw in **canvas** space, and if you
ever want "proper" geometry (angles that increase counter-clockwise, etc.) you
convert to **math** space. For this project we live almost entirely in canvas
space.

### 1.4 Transforms between the spaces

**Client → Canvas** (needed to turn a mouse position into a pixel):

```
rect   = canvas.getBoundingClientRect()      // canvas position/size on screen
scaleX = canvas.width  / rect.width          // framebuffer px per CSS px
scaleY = canvas.height / rect.height

x_canvas = (event.clientX - rect.left) * scaleX
y_canvas = (event.clientY - rect.top)  * scaleY
```

The `scaleX/scaleY` factor exists because the canvas element can be **displayed**
at one size (CSS) while its **backing store** (the actual framebuffer) is another.
If they differ, every mouse coordinate must be rescaled or your drawing will be
offset from your cursor.

**Canvas ↔ Math/NDC** (only if you want a "y-up" world):

```
x_px = (x_ndc + 1) / 2 * W
y_px = (1 - y_ndc) / 2 * H
```

The `1 - y` is exactly the y-flip that converts "up is positive" into "down is
positive".

---

## 2. The math of a straight line

### 2.1 The ideal line

A line is a set of infinitely many points. Given two endpoints `P0 = (x0, y0)`
and `P1 = (x1, y1)`, every point `P` on the segment satisfies the **parametric
form**:

```
P(t) = P0 + t * (P1 - P0),     t in [0, 1]
```

Written out:

```
x(t) = x0 + t * (x1 - x0)
y(t) = y0 + t * (y1 - y0)
```

`t = 0` gives `P0`, `t = 1` gives `P1`, and `t = 1/2` gives the midpoint. This is
the cleanest, division-safe, orientation-safe description of a line segment. It
has no vertical-line problem and no infinite slopes.

### 2.2 Slope-intercept form, and why it is not enough

The familiar school form is:

```
y = m * x + b
m = (y1 - y0) / (x1 - x0)      // slope
b = y0 - m * x0                // intercept
```

It is fine for intuition but **bad for rasterization**:

- **Vertical lines** have `x1 = x0`, so `m = infinity`. The formula simply does
  not apply and needs a special case.
- It computes `y` from `x`, which only works when the line is "more horizontal
  than vertical". When the line is steep, you need the inverse form `x = f(y)`.
- It uses a **floating-point division per point**, which is slower than necessary
  and accumulates error.

So we use it to *understand* lines, but we do not use it to *draw* them.

### 2.3 The rasterization problem

A real line is **continuous**. A canvas is **discrete** — a grid of whole-number
positions. We cannot light up "the point at y = 3.47"; we can only choose the
nearest grid cell. Rasterization is the act of choosing, for a continuum of
points, the finite set of pixels that best represents the line.

Two guiding properties a good algorithm must have:

- **No gaps.** Adjacent pixels in the line should touch (edge-adjacent, or at
  worst corner-adjacent), otherwise the line looks dotted.
- **Minimal thickness.** The line should be ~1 pixel wide, not a fat smear.

---

## 3. Line rasterization algorithms

We build up from the worst idea to the classic best idea. Understanding the
progression is the actual point of this section.

### 3.1 Naive floating-point sampling

Idea: use the parametric form, sample `t` in many small steps, and round.

Weaknesses:

- How many steps? Enough to avoid gaps means roughly `max(|dx|, |dy|)` steps —
  which is exactly what DDA formalizes.
- Per-step floating-point multiply and round — error-prone and slower than the
  integer algorithms.

### 3.2 DDA (Digital Differential Analyzer)

DDA fixes the "how many steps" problem deterministically.

Let `dx = x1 - x0`, `dy = y1 - y0`, and `steps = max(|dx|, |dy|)`. Then:

```
xStep = dx / steps
yStep = dy / steps
```

Start at `(x0, y0)`. On each of the `steps + 1` iterations:

1. Plot `(round(x), round(y))`.
2. `x += xStep`
3. `y += yStep`

Properties:

- Guarantees **one pixel of movement along the major axis** per step → no gaps.
- It is an **incremental** algorithm: it adds a constant instead of recomputing a
  formula, which is the key speed idea reused by Bresenham.
- It still uses **floating point**, so accumulated rounding error can make long
  lines drift by a pixel.
- Division `dx/steps`, `dy/steps` is done once (not per pixel), so it is already
  much better than naive.

DDA is the conceptual bridge. Bresenham is DDA rebuilt with **pure integers**.

### 3.3 Bresenham / midpoint line algorithm (the target)

Bresenham's algorithm answers a simple question **with integer arithmetic only**:
*at each column, should the next pixel be to the right (same row), or diagonally
up/right (next row)?*

Assume the **first octant**: `x1 > x0`, and `0 <= dy <= dx` (a shallow,
left-to-right, descending-in-math/rising-in-screen line). We step `x` by 1 each
time; the only decision is whether `y` also increments by 1.

**Step 1 — an implicit equation for the line.**

Instead of `y = mx + b`, use the function:

```
F(x, y) = dy * (x - x0) - dx * (y - y0)
```

This is zero exactly *on* the line, positive on one side, negative on the other.
Check: `F(x0, y0) = 0`, and `F(x1, y1) = dy*dx - dx*dy = 0`. 

**Step 2 — the midpoint test.**

Suppose we just plotted `(xp, yp)`. Next column is `xp + 1`. The two candidate
pixels are:

- `E` = east = `(xp + 1, yp)`
- `NE` = northeast = `(xp + 1, yp + 1)`

The true line passes somewhere between them. Take the **midpoint**
`M = (xp + 1, yp + 1/2)` and evaluate `F(M)`:

- If the line passes **below** M, choose `E`.
- If the line passes **above** M, choose `NE`.

So the **sign of `F(M)`** is the entire decision.

**Step 3 — remove the fraction.**

Evaluating `F(M)` gives halves. Multiply the whole decision by 2 to keep it
integral. Define the **decision parameter** `D`:

```
D = 2 * F(M)
  = 2*dy*(xp + 1 - x0) - dx*(2*yp + 1 - 2*y0)
```

At the start `(x0, y0)` this reduces to:

```
D0 = 2*dy - dx
```

**Step 4 — incremental update.**

We never recompute `D`; we update it. When we advance one column:

- If we chose `E` (y unchanged): `D` increases by `2*dy`.
- If we chose `NE` (y incremented): `D` increases by `2*(dy - dx)`.

Because `D0 = 2dy - dx` and we know `0 <= dy <= dx`, the rule becomes:

```
if D < 0:   plot (x+1, y);      D = D + 2*dy
else:       plot (x+1, y+1);    D = D + 2*(dy - dx)
```

This is the whole algorithm: **no division, no floats, one add and one compare
per pixel.** That is why it is the standard.

### 3.4 Generalizing to all 8 octants

The derivation assumed the first octant. Rather than re-deriving eight times, use
**symmetry**:

1. Compute the direction of travel: `sx = sign(x1 - x0)`, `sy = sign(y1 - y0)`.
2. Work with magnitudes `adx = |dx|`, `ady = |dy|`.
3. Choose the **driving axis**:
   - If `adx >= ady` (shallow): step along **x**; Bresenham decides when **y**
     jumps. Use the algorithm above, then apply `sx` and `sy` signs when writing
     pixels.
   - If `ady > adx` (steep): step along **y**; the decision now controls **x**.
     This is just the shallow algorithm with `x` and `y` swapped.
4. **Vertical lines** (`adx = 0`) and **horizontal lines** (`ady = 0`) fall out of
   the general case, as long as the driving-axis choice handles `adx = 0`
   gracefully.

The key insight: a line is symmetric under reflection and swapping axes, so one
algorithm plus sign/swap bookkeeping covers every possible line.

### 3.5 Comparison

| Algorithm | Arithmetic | Gaps? | Drift? | Notes |
|-----------|-----------|-------|--------|-------|
| Naive parametric | float, per-point | possible | yes | simple, slow |
| DDA | float incremental | no | possible | good teaching step |
| Bresenham | **integer only** | no | no | the standard |

### 3.6 Optional: anti-aliasing (Xiaolin Wu)

Bresenham lines look "stair-stepped" (aliased) on shallow slopes. Wu's algorithm
softens this by drawing **two pixels per column**, each with a partial alpha:

```
frac   = fractional part of the true y at column x
pixel (x, floor(y))   gets intensity (1 - frac)
pixel (x, floor(y)+1) gets intensity (frac)
```

Those intensities are **blended** with what is already on the canvas using alpha
compositing:

```
result = src_color * alpha + dst_color * (1 - alpha)
```

This is a nice later upgrade. It is not required for a first working line tool.

### 3.7 Optional: line thickness

Two common approaches:

- **Stamp parallel lines.** Draw several Bresenham lines offset perpendicular to
  the direction by `-t/2 ... +t/2` and round. Cheap, but corners between segments
  can look gappy.
- **Distance field (more correct).** For every pixel `P` in the bounding box of
  the segment, compute its perpendicular distance to the line and fill it if the
  distance is `<= t/2`. The distance from point `P` to the line through `A, B` is:

  ```
  d = |(B - A) × (P - A)| / |B - A|
  ```

  where `×` is the 2D cross product `(ux * vy - uy * vx)`. This naturally gives
  round caps and is what you would use for a "brush size" feature.

---

## 4. Interaction math (mouse / touch → pixels)

### 4.1 Pointer events

Use the **Pointer Events** API (`pointerdown`, `pointermove`, `pointerup`) rather
than mouse events: it unifies mouse, pen, and touch. Call
`element.setPointerCapture(event.pointerId)` on `pointerdown` so that moves are
still delivered even if the cursor leaves the canvas mid-stroke.

### 4.2 Client → canvas transform

Every pointer event gives **client** coordinates. Convert them with the formula
from §1.4. If your canvas is fixed `800×800` in both CSS and backing store, the
scale is `1` and you can skip the ratio — but write the general formula anyway so
it survives later changes (see §7, device pixel ratio).

For a good drawing feel, **round to the nearest integer pixel** at the moment you
commit a point, and optionally coerce decimals to `.5` if you later switch the
pixel model so that integer coordinates denote pixel *centers*.

### 4.3 The "rubber band" preview

While the pointer is down, you want the line to follow the cursor, but you must
**not** smear old preview positions onto the canvas. The clean pattern:

- Keep a **committed** framebuffer: all lines that are finished.
- On each move, **reset the display framebuffer to the committed one**, then draw
  a single preview line from the anchor point to the current cursor.
- On `pointerup`, draw the final line *into the committed buffer* and drop the
  preview.

This means you need either **two buffers** (committed + display) or the ability
to **replay** all committed lines each frame. Two buffers is simpler and is what
we will assume.

---

# PART B — HOW THE CODE WILL WORK

## 5. Repo architecture recap

This project is a Rust + WebAssembly + TypeScript stack:

```
Arch/
├── engine/                 # pure Rust library, no browser knowledge
│   └── src/lib.rs          # the real logic (currently a Counter)
├── frontend/
│   ├── wrapper/            # Rust crate that exposes engine to JS via wasm-bindgen
│   │   └── src/lib.rs      # #[wasm_bindgen] types (currently CounterWrapper)
│   ├── wrapper/pkg/        # generated JS + .d.ts + .wasm (build output)
│   └── src/
│       ├── main.ts         # app entry (currently the Counter UI)
│       └── wasm-loader.ts  # awaits wasm init once
└── Cargo.toml              # workspace: engine + frontend/wrapper
```

Layering rule to preserve:

- **engine** — knows nothing about JS, HTML, or wasm-bindgen. Pure data + math.
  This is where `Framebuffer`, `set_pixel`, and `draw_line` belong.
- **wrapper** — thin. It only wraps engine types with `#[wasm_bindgen]` and
  translates the wasm boundary. No graphics math here.
- **frontend/src** — DOM, canvas element, pointer events, and the wasm bridge.

## 6. Two possible designs

**Design A — TypeScript only (use the browser's built-in lines).**
Create a canvas, get `CanvasRenderingContext2D`, and on pointer drag call
`beginPath / moveTo / lineTo / stroke`. The browser's native rasterizer does the
Bresenham work for you. This is the fastest path to a working tool and is worth
knowing, **but it defeats the purpose of this repo** — the Rust engine would do
nothing.

**Design B — Rust engine owns the framebuffer (recommended here).**
The engine allocates a `W × H × 4` byte buffer, implements `set_pixel` and
`draw_line` (Bresenham) itself, and the frontend displays that buffer on the
canvas. This is the real "build a graphics engine" path, and it matches the
existing Counter-through-wasm pattern. Everything below assumes Design B.

## 7. Data flow (Design B)

```
pointer event  ──►  main.ts: client→canvas transform
                        │
                        ▼
               engine draw_line(x0,y0,x1,y1,color)
                        │  (via wasm wrapper)
                        ▼
               engine framebuffer: Vec<u8> (RGBA rows)
                        │
                        ▼
   JS reads wasm memory as Uint8ClampedArray (zero copy)
                        │
                        ▼
   new ImageData(view, W, H) ──► ctx.putImageData(imageData, 0, 0)
                        │
                        ▼
                    pixels on screen
```

The Rust engine never touches the DOM; the TS layer never implements line math.
That separation is the whole design.

## 8. The engine layer (Rust)

Add to `engine/src/lib.rs` (conceptually — a `Framebuffer` type):

- **Fields:** `width: u32`, `height: u32`, `pixels: Vec<u8>` (length
  `width * height * 4`).
- **`new(width, height)`** — allocate the buffer, initialize all pixels (e.g.
  opaque white or transparent).
- **`clear(color)`** — overwrite every pixel with a color.
- **`set_pixel(x, y, color)`** — bounds-check `0 <= x < W`, `0 <= y < H`, compute
  `(y * W + x) * 4`, write RGBA. Silently ignore out-of-bounds (or clamp) so
  algorithms can call it freely.
- **`draw_line(x0, y0, x1, y1, color)`** — the Bresenham implementation from
  §3.3–§3.4. All integers. Handle the shallow/steep split and sign steps.
- **(optional) `draw_line_aa(...)`** — Wu's algorithm (§3.6) with alpha blending
  that reads the destination pixel before writing.

Represent color as 4 bytes or a packed `u32`. A small `Color` struct/type keeps
the API readable. No loops should allocate; the buffer is allocated once.

## 9. The wrapper layer (wasm-bindgen)

The wrapper's job is to make the engine callable from JS without exposing
Rust-only types (like `Vec<u8>` slices, references, lifetimes). Add a
`FramebufferWrapper` alongside `CounterWrapper`:

- **`#[wasm_bindgen] pub struct FramebufferWrapper { inner: Framebuffer }`**
- **`#[wasm_bindgen(constructor)] new(width, height)`**
- **`clear(r, g, b, a)`**
- **`set_pixel(x, y, r, g, b, a)`**
- **`draw_line(x0, y0, x1, y1, r, g, b, a)`**
- **Expose the buffer to JS via a pointer.** The idiomatic zero-copy pattern:
  - `ptr(&self) -> *const u8` returning `self.inner.pixels.as_ptr()`.
  - `len(&self) -> usize` returning `self.inner.pixels.len()`.
  - Optionally `width()` / `height()`.

  wasm-bindgen can hand these pointers to JS as numbers; the JS side then builds
  a typed-array view. Alternatively return `Vec<u8>` (a copy) for simplicity
  first, and optimize to the pointer later.

After changing the wrapper you must **rebuild** it with `wasm-pack build` (check
the wrapper's build script / `pkg` output); the generated
`arch_wasm_wrapper.js` / `.d.ts` in `wrapper/pkg` is what `wasm-loader.ts`
imports. The generated `.d.ts` will tell you the exact JS signatures.

## 10. The frontend layer (TypeScript)

Replace the Counter UI in `src/main.ts` with a line tool. Responsibilities:

1. **Create the DOM.** A `<canvas>` of fixed size, and enough markup for a
   container. Set both `canvas.width`/`canvas.height` (backing store) and, via
   CSS or inline style, the displayed size. For a truly fixed canvas make them
   equal, e.g. `800×800`.
2. **Init wasm.** `await initWasm()` (already provided).
3. **Create the engine framebuffer.** `new FramebufferWrapper(W, H)`.
4. **Get the 2D context:** `canvas.getContext('2d')`.
5. **Bridge pixels out of wasm (each redraw):**
   - Read the wasm memory export: `wasmMemory.buffer`.
   - Build a view:
     `new Uint8ClampedArray(memoryBuffer, framebuffer.ptr(), framebuffer.len())`.
   - Wrap it: `new ImageData(view, W, H)`.
   - Blit: `ctx.putImageData(imageData, 0, 0)`.
6. **Pointer handling:**
   - `pointerdown`: convert coords (§4.2), store as the **anchor**; optionally
     capture the pointer.
   - `pointermove` (while down): redraw committed lines + a preview line from
     anchor to current point, then blit.
   - `pointerup`: commit the final line into the engine, clear the preview, blit.
7. **Render scheduling:** don't blit synchronously on every event if the pointer
   fires faster than the display refreshes. Mark "dirty" and blit inside
   `requestAnimationFrame`. (Fine to start synchronous; optimize later.)

The TS side contains **no** Bresenham code — it only converts coordinates and
shuttles bytes.

## 11. The zero-copy pixel bridge (important details)

This is the trickiest part, so it deserves explicit notes:

- `WebAssembly.Memory` exports a single `ArrayBuffer` called `memory.buffer`.
  The framebuffer bytes live inside it at `ptr` for `len` bytes.
- A typed array (`Uint8ClampedArray`) created over that buffer is a **view**, not
  a copy — writes in Rust are immediately visible. This is why we prefer the
  pointer over returning a copied `Vec`.
- **Memory growth invalidates old views.** If Rust ever grows wasm memory, any
  previously created typed array (and any `ImageData` wrapping it) becomes
  detached. **Recreate the view every frame**, or at minimum after any call that
  might reallocate. Since the framebuffer is allocated once and never resized,
  growth is unlikely — but recreate defensively.
- `ImageData` requires exactly `W * H * 4` bytes, in RGBA, top-left origin. Our
  engine layout already matches, so no conversion is needed.
- `putImageData` **replaces** pixels and ignores canvas transforms/blending. That
  is why the redraw loop is "clear/rebuild → putImageData", not "draw on top".

## 12. Keeping the preview without smearing

Two framebuffers in the engine (or two `FramebufferWrapper`s):

- **committed** — grows by one line on each `pointerup`.
- **display** — rebuilt every move as: copy of committed, then the preview line.

Simplest implementation: since committed holds only finished lines, you can
instead **replay all committed lines + the preview line from scratch** on every
move. That avoids a copy but costs `O(number of lines)` per move. For a first
version either is fine; the two-buffer copy is `O(W*H)` per move but independent
of line count.

---

# 13. Implementation checklist (what you have to do)

Engine (`engine/src/lib.rs`):
- [ ] Add a `Framebuffer` struct with `width`, `height`, `pixels: Vec<u8>`.
- [ ] Implement `new`, `clear`, `set_pixel` (bounds-checked, row-major indexing).
- [ ] Implement `draw_line` with Bresenham: sign steps, shallow/steep split,
      `D0 = 2*dy - dx`, integer updates. Handle vertical/horizontal lines.
- [ ] (optional) Wu anti-aliasing with alpha blending.
- [ ] (optional) thickness via distance field.

Wrapper (`frontend/wrapper/src/lib.rs`):
- [ ] Add `FramebufferWrapper` with `new`, `clear`, `set_pixel`, `draw_line`.
- [ ] Expose `ptr()`, `len()`, `width()`, `height()` for zero-copy access.
- [ ] Rebuild the wasm package with `wasm-pack` so `wrapper/pkg` is current.

Frontend (`frontend/src/main.ts`):
- [ ] Replace the Counter UI with a fixed-size `<canvas>`.
- [ ] `await initWasm()`, construct the framebuffer, get the 2D context.
- [ ] Write the client→canvas coordinate transform.
- [ ] Wire pointerdown / pointermove / pointerup with an anchor point.
- [ ] Write a `render()` that builds `ImageData` from the wasm view and calls
      `putImageData`.
- [ ] Implement the committed + preview redraw pattern.
- [ ] (optional) Move rendering into `requestAnimationFrame`.

Verify:
- [ ] `npm run dev` (in `frontend/`) and draw lines in all directions, including
      perfectly horizontal, vertical, steep, and shallow.
- [ ] `npm run build` type-checks (`tsc`) cleanly.

---

# 14. Gotchas to remember

- **y points down** on a canvas. "Downward slope" and "upward slope" are
  screen-relative.
- **Vertical lines** have no slope — never use `y = mx + b` to draw.
- **Client vs canvas coordinates** must be rescaled if CSS size ≠ backing-store
  size, or if you use a **devicePixelRatio**-scaled canvas (Retina). If you scale
  for DPR, do it on the *backing store* (`canvas.width = cssW * dpr`) and keep the
  engine buffer in device pixels.
- **Recreate the typed-array view after any wasm memory growth** (§11).
- **`putImageData` replaces, doesn't blend** — redraw the whole frame.
- **Preview smearing** is prevented only by replaying committed content each
  move (§12).
- **Rebuild wasm** whenever you change the Rust wrappers, or JS will call stale
  functions.
- Bresenham's **decision sign/increment direction** depends on the assumed
  octant; derive once, then handle the other octants purely by **sign flips and
  axis swaps** (§3.4), not by rewriting the algorithm.
