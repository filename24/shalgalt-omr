import type { TemplatePoint } from "$lib/types/template";

/**
 * Pixel rect on the Konva stage where the contained backdrop image actually
 * paints after letterboxing (object-fit: contain semantics). All marker /
 * bubble positions are stored in `[0,1]` normalized space relative to *this*
 * rect — never the stage itself — so that resizing the window does not move
 * any logical coordinate.
 */
export interface BackdropRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

/**
 * Letterbox an `imgW × imgH` image inside a `stageW × stageH` viewport using
 * object-fit: contain. Returns the pixel rect where the image renders. When
 * either image dimension is zero (image not yet loaded), returns a zero-area
 * rect centered in the stage.
 */
export function fitContain(
  stageW: number,
  stageH: number,
  imgW: number,
  imgH: number,
): BackdropRect {
  if (imgW <= 0 || imgH <= 0 || stageW <= 0 || stageH <= 0) {
    return { x: stageW / 2, y: stageH / 2, width: 0, height: 0 };
  }
  const stageRatio = stageW / stageH;
  const imgRatio = imgW / imgH;
  let width: number;
  let height: number;
  if (imgRatio > stageRatio) {
    width = stageW;
    height = stageW / imgRatio;
  } else {
    height = stageH;
    width = stageH * imgRatio;
  }
  return {
    x: (stageW - width) / 2,
    y: (stageH - height) / 2,
    width,
    height,
  };
}

/** A4 portrait — 210 × 297 mm. Used as the synthetic backdrop ratio. */
export const A4_PORTRAIT_W = 210;
export const A4_PORTRAIT_H = 297;

/**
 * Same as `fitContain` but falls back to an A4-portrait rect when no image is
 * loaded. Used when rendering presets without an imported backdrop, so the
 * bubbles still appear at meaningful coordinates against a synthetic page.
 */
export function fitContainOrA4(
  stageW: number,
  stageH: number,
  imgW: number,
  imgH: number,
): BackdropRect {
  if (imgW > 0 && imgH > 0) {
    return fitContain(stageW, stageH, imgW, imgH);
  }
  return fitContain(stageW, stageH, A4_PORTRAIT_W, A4_PORTRAIT_H);
}

/** Convert a normalized template point to absolute pixel coordinates on the stage. */
export function toCanvasPx(
  p: TemplatePoint,
  r: BackdropRect,
): { x: number; y: number } {
  return {
    x: r.x + p.x * r.width,
    y: r.y + p.y * r.height,
  };
}

/**
 * Convert a stage pixel coordinate back into normalized space. Caller
 * typically chains this through `clampNorm` because Konva drag events can
 * place the node outside the backdrop rect.
 */
export function toNormalized(
  px: { x: number; y: number },
  r: BackdropRect,
): TemplatePoint {
  if (r.width <= 0 || r.height <= 0) return { x: 0, y: 0 };
  return {
    x: (px.x - r.x) / r.width,
    y: (px.y - r.y) / r.height,
  };
}

/** Force a normalized point into the unit square. */
export function clampNorm(p: TemplatePoint): TemplatePoint {
  return {
    x: Math.min(1, Math.max(0, p.x)),
    y: Math.min(1, Math.max(0, p.y)),
  };
}
