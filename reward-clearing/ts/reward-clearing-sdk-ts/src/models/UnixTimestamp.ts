/**
 * A moment in time: whole seconds since 1970-01-01 UTC. The program stores
 * it as a `u64`.
 */
export interface UnixTimestamp {
  readonly seconds: bigint
}
