// Generated from an Anchor IDL. Do not edit.

import { decodeData, encodeData, hasDiscriminator } from "./codecs.js"
import * as models from "./models.js"

// account codecs ----------------------------------------------------

export const LINEAR_CHARACTERISTIC_PARAM_ACCOUNT_DISCRIMINATOR =
  Uint8Array.from([168, 158, 201, 134, 101, 136, 37, 119])

export const decodeLinearCharacteristicParamAccount = (
  data: Uint8Array,
): models.LinearCharacteristicParam =>
  decodeData(
    models.LINEAR_CHARACTERISTIC_PARAM_SCHEMA,
    LINEAR_CHARACTERISTIC_PARAM_ACCOUNT_DISCRIMINATOR,
    data,
  )

export const encodeLinearCharacteristicParamAccount = (
  value: models.LinearCharacteristicParam,
): Uint8Array =>
  encodeData(
    models.LINEAR_CHARACTERISTIC_PARAM_SCHEMA,
    LINEAR_CHARACTERISTIC_PARAM_ACCOUNT_DISCRIMINATOR,
    value,
  )

export type DecodedMayfAvmEngineLinearAccount = {
  name: "linearCharacteristicParam"
  data: models.LinearCharacteristicParam
}

export const decodeMayfAvmEngineLinearAccount = (
  data: Uint8Array,
): DecodedMayfAvmEngineLinearAccount | null => {
  if (
    hasDiscriminator(data, LINEAR_CHARACTERISTIC_PARAM_ACCOUNT_DISCRIMINATOR)
  ) {
    return {
      name: "linearCharacteristicParam",
      data: decodeLinearCharacteristicParamAccount(data),
    }
  }
  return null
}
