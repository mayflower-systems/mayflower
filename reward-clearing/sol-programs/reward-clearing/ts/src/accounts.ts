// Generated from an Anchor IDL. Do not edit.

import { decodeData, encodeData, hasDiscriminator } from "./codecs.js"
import * as models from "./models.js"

// account codecs ----------------------------------------------------

export const BIDDER_POSITION_ACCOUNT_DISCRIMINATOR = Uint8Array.from([
  86, 37, 176, 141, 39, 7, 120, 255,
])

export const decodeBidderPositionAccount = (
  data: Uint8Array,
): models.BidderPosition =>
  decodeData(
    models.BIDDER_POSITION_SCHEMA,
    BIDDER_POSITION_ACCOUNT_DISCRIMINATOR,
    data,
  )

export const encodeBidderPositionAccount = (
  value: models.BidderPosition,
): Uint8Array =>
  encodeData(
    models.BIDDER_POSITION_SCHEMA,
    BIDDER_POSITION_ACCOUNT_DISCRIMINATOR,
    value,
  )

export const CHANNEL_ACCOUNT_DISCRIMINATOR = Uint8Array.from([
  49, 159, 99, 106, 220, 87, 219, 88,
])

export const decodeChannelAccount = (data: Uint8Array): models.Channel =>
  decodeData(models.CHANNEL_SCHEMA, CHANNEL_ACCOUNT_DISCRIMINATOR, data)

export const encodeChannelAccount = (value: models.Channel): Uint8Array =>
  encodeData(models.CHANNEL_SCHEMA, CHANNEL_ACCOUNT_DISCRIMINATOR, value)

export const CHANNEL_EPOCH_ACCOUNT_DISCRIMINATOR = Uint8Array.from([
  42, 194, 102, 118, 28, 161, 115, 40,
])

export const decodeChannelEpochAccount = (
  data: Uint8Array,
): models.ChannelEpoch =>
  decodeData(
    models.CHANNEL_EPOCH_SCHEMA,
    CHANNEL_EPOCH_ACCOUNT_DISCRIMINATOR,
    data,
  )

export const encodeChannelEpochAccount = (
  value: models.ChannelEpoch,
): Uint8Array =>
  encodeData(
    models.CHANNEL_EPOCH_SCHEMA,
    CHANNEL_EPOCH_ACCOUNT_DISCRIMINATOR,
    value,
  )

export const CHANNEL_GROUP_ACCOUNT_DISCRIMINATOR = Uint8Array.from([
  36, 177, 239, 118, 135, 238, 130, 4,
])

export const decodeChannelGroupAccount = (
  data: Uint8Array,
): models.ChannelGroup =>
  decodeData(
    models.CHANNEL_GROUP_SCHEMA,
    CHANNEL_GROUP_ACCOUNT_DISCRIMINATOR,
    data,
  )

export const encodeChannelGroupAccount = (
  value: models.ChannelGroup,
): Uint8Array =>
  encodeData(
    models.CHANNEL_GROUP_SCHEMA,
    CHANNEL_GROUP_ACCOUNT_DISCRIMINATOR,
    value,
  )

export const EPOCH_ACCOUNT_DISCRIMINATOR = Uint8Array.from([
  93, 83, 120, 89, 151, 138, 152, 108,
])

export const decodeEpochAccount = (data: Uint8Array): models.Epoch =>
  decodeData(models.EPOCH_SCHEMA, EPOCH_ACCOUNT_DISCRIMINATOR, data)

export const encodeEpochAccount = (value: models.Epoch): Uint8Array =>
  encodeData(models.EPOCH_SCHEMA, EPOCH_ACCOUNT_DISCRIMINATOR, value)

export const ORGANIZATION_ACCOUNT_DISCRIMINATOR = Uint8Array.from([
  145, 38, 152, 251, 91, 57, 118, 160,
])

export const decodeOrganizationAccount = (
  data: Uint8Array,
): models.Organization =>
  decodeData(
    models.ORGANIZATION_SCHEMA,
    ORGANIZATION_ACCOUNT_DISCRIMINATOR,
    data,
  )

export const encodeOrganizationAccount = (
  value: models.Organization,
): Uint8Array =>
  encodeData(
    models.ORGANIZATION_SCHEMA,
    ORGANIZATION_ACCOUNT_DISCRIMINATOR,
    value,
  )

export type DecodedMayfRewardClearingAccount =
  | { name: "bidderPosition"; data: models.BidderPosition }
  | { name: "channel"; data: models.Channel }
  | { name: "channelEpoch"; data: models.ChannelEpoch }
  | { name: "channelGroup"; data: models.ChannelGroup }
  | { name: "epoch"; data: models.Epoch }
  | { name: "organization"; data: models.Organization }

export const decodeMayfRewardClearingAccount = (
  data: Uint8Array,
): DecodedMayfRewardClearingAccount | null => {
  if (hasDiscriminator(data, BIDDER_POSITION_ACCOUNT_DISCRIMINATOR)) {
    return { name: "bidderPosition", data: decodeBidderPositionAccount(data) }
  }
  if (hasDiscriminator(data, CHANNEL_ACCOUNT_DISCRIMINATOR)) {
    return { name: "channel", data: decodeChannelAccount(data) }
  }
  if (hasDiscriminator(data, CHANNEL_EPOCH_ACCOUNT_DISCRIMINATOR)) {
    return { name: "channelEpoch", data: decodeChannelEpochAccount(data) }
  }
  if (hasDiscriminator(data, CHANNEL_GROUP_ACCOUNT_DISCRIMINATOR)) {
    return { name: "channelGroup", data: decodeChannelGroupAccount(data) }
  }
  if (hasDiscriminator(data, EPOCH_ACCOUNT_DISCRIMINATOR)) {
    return { name: "epoch", data: decodeEpochAccount(data) }
  }
  if (hasDiscriminator(data, ORGANIZATION_ACCOUNT_DISCRIMINATOR)) {
    return { name: "organization", data: decodeOrganizationAccount(data) }
  }
  return null
}
