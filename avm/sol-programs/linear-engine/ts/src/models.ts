// Generated from an Anchor IDL. Do not edit.

import type { Schema } from "borsh"

// types -------------------------------------------------------------

export interface CalculateSpansArgs {
  spans: ReadonlyArray<SpanRequest>
}

export interface CreateLinearEngineArgs {
  slopeQ64x64: bigint
}

export interface LinearCharacteristicParam {
  /**
   * Positive, immutable slope of `f(x) = slope * x`, on the `2^-64` grid.
   */
  slopeQ64x64: bigint
}

export interface Q64x64Bounds {
  lower: bigint
  upper: bigint
}

export interface SpanRequest {
  loQ64x64: bigint
  /**
   * Zero requests a point evaluation with zero area.
   */
  widthQ64x64: bigint
}

export interface SpanResponse {
  yAtLo: Q64x64Bounds
  yAtHi: Q64x64Bounds
  area: Q64x64Bounds
}

// Borsh schemas -----------------------------------------------------

export const SPAN_REQUEST_SCHEMA: Schema = {
  struct: { loQ64x64: "u128", widthQ64x64: "u128" },
}
export const CALCULATE_SPANS_ARGS_SCHEMA: Schema = {
  struct: { spans: { array: { type: SPAN_REQUEST_SCHEMA } } },
}
export const CREATE_LINEAR_ENGINE_ARGS_SCHEMA: Schema = {
  struct: { slopeQ64x64: "u128" },
}
export const LINEAR_CHARACTERISTIC_PARAM_SCHEMA: Schema = {
  struct: { slopeQ64x64: "u128" },
}
export const Q64X64_BOUNDS_SCHEMA: Schema = {
  struct: { lower: "u128", upper: "u128" },
}
export const SPAN_RESPONSE_SCHEMA: Schema = {
  struct: {
    yAtLo: Q64X64_BOUNDS_SCHEMA,
    yAtHi: Q64X64_BOUNDS_SCHEMA,
    area: Q64X64_BOUNDS_SCHEMA,
  },
}
