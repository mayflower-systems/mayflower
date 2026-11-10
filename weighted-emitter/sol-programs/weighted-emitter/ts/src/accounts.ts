// Generated from an Anchor IDL. Do not edit.

import { decodeData, encodeData, hasDiscriminator } from "./codecs.js"
import * as models from "./models.js"

// account codecs ----------------------------------------------------

export const EMITTER_ACCOUNT_DISCRIMINATOR = Uint8Array.from([
  4, 60, 118, 104, 34, 153, 156, 218,
])

export const decodeEmitterAccount = (data: Uint8Array): models.Emitter =>
  decodeData(models.EMITTER_SCHEMA, EMITTER_ACCOUNT_DISCRIMINATOR, data)

export const encodeEmitterAccount = (value: models.Emitter): Uint8Array =>
  encodeData(models.EMITTER_SCHEMA, EMITTER_ACCOUNT_DISCRIMINATOR, value)

export const GAUGE_ACCOUNT_DISCRIMINATOR = Uint8Array.from([
  9, 19, 249, 189, 158, 171, 226, 205,
])

export const decodeGaugeAccount = (data: Uint8Array): models.Gauge =>
  decodeData(models.GAUGE_SCHEMA, GAUGE_ACCOUNT_DISCRIMINATOR, data)

export const encodeGaugeAccount = (value: models.Gauge): Uint8Array =>
  encodeData(models.GAUGE_SCHEMA, GAUGE_ACCOUNT_DISCRIMINATOR, value)

export const VOTE_POSITION_ACCOUNT_DISCRIMINATOR = Uint8Array.from([
  121, 54, 167, 23, 147, 96, 61, 209,
])

export const decodeVotePositionAccount = (
  data: Uint8Array,
): models.VotePosition =>
  decodeData(
    models.VOTE_POSITION_SCHEMA,
    VOTE_POSITION_ACCOUNT_DISCRIMINATOR,
    data,
  )

export const encodeVotePositionAccount = (
  value: models.VotePosition,
): Uint8Array =>
  encodeData(
    models.VOTE_POSITION_SCHEMA,
    VOTE_POSITION_ACCOUNT_DISCRIMINATOR,
    value,
  )

export type DecodedMayfWeightedEmitterAccount =
  | { name: "emitter"; data: models.Emitter }
  | { name: "gauge"; data: models.Gauge }
  | { name: "votePosition"; data: models.VotePosition }

export const decodeMayfWeightedEmitterAccount = (
  data: Uint8Array,
): DecodedMayfWeightedEmitterAccount | null => {
  if (hasDiscriminator(data, EMITTER_ACCOUNT_DISCRIMINATOR)) {
    return { name: "emitter", data: decodeEmitterAccount(data) }
  }
  if (hasDiscriminator(data, GAUGE_ACCOUNT_DISCRIMINATOR)) {
    return { name: "gauge", data: decodeGaugeAccount(data) }
  }
  if (hasDiscriminator(data, VOTE_POSITION_ACCOUNT_DISCRIMINATOR)) {
    return { name: "votePosition", data: decodeVotePositionAccount(data) }
  }
  return null
}
