import {
  createContext,
  createElement,
  useContext,
  useEffect,
  useSyncExternalStore,
  type ReactNode,
} from "react"
import type { Dispatch, Kernel } from "./kernel.js"

/** The React bindings for the kernels of one `State` and `Msg` pair. */
export type ElmContext<State, Msg> = {
  /**
   * Gives `kernel` to the hooks of the components below. Starts the kernel
   * when it mounts and stops it when it unmounts.
   */
  readonly Provider: (props: {
    readonly kernel: Kernel<State, Msg>
    readonly children: ReactNode
  }) => ReactNode
  /**
   * Returns `select(state)`. Re-renders the component when that value changes,
   * as compared by `Object.is`.
   *
   * `select` must return the same value for the same state: read a field, or
   * compute a primitive. A selector that builds a new object or array on each
   * call never compares equal, and React aborts the render with "Maximum
   * update depth exceeded". To combine values, call `useSelect` once for each
   * value and merge the results with `useMemo`.
   */
  readonly useSelect: <Selected>(select: (state: State) => Selected) => Selected
  /** Returns the kernel's `dispatch`. Its identity never changes. */
  readonly useDispatch: () => Dispatch<Msg>
}

/**
 * Creates the React bindings for one `State` and `Msg` pair. Call it once, at
 * module scope, in a module that exports no component. React Fast Refresh
 * reloads the page when a component module has other exports, and a reload
 * resets the kernel.
 */
export const createElmContext = <State, Msg>(): ElmContext<State, Msg> => {
  const Context = createContext<Kernel<State, Msg> | null>(null)

  const useKernel = (): Kernel<State, Msg> => {
    const kernel = useContext(Context)
    if (kernel === null) {
      throw new Error(
        "No kernel in context. Render this component under its Provider.",
      )
    }
    return kernel
  }

  return {
    Provider: ({ kernel, children }) => {
      // `start` returns its own stop function, which is the cleanup.
      // StrictMode starts, stops, and starts again. The commands of `init`
      // still launch once. The subscriptions restart.
      useEffect(kernel.start, [kernel])
      return createElement(Context, { value: kernel }, children)
    },
    useSelect: (select) => {
      const kernel = useKernel()
      return useSyncExternalStore(kernel.subscribe, () =>
        select(kernel.getState()),
      )
    },
    useDispatch: () => useKernel().dispatch,
  }
}
