/**
 * The outcome of an operation that can fail for a reason the caller should
 * expect. Such a failure is a returned value. This package throws only for a
 * defect: a bug in the calling program.
 */
export type Result<A, E> =
  | { readonly ok: true; readonly value: A }
  | { readonly ok: false; readonly error: E }

export const ok = <A>(value: A): Result<A, never> => ({ ok: true, value })

export const err = <const E>(error: E): Result<never, E> => ({
  ok: false,
  error,
})
