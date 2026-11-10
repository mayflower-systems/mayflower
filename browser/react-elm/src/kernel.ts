/** Sends a message to `update`. */
export type Dispatch<Msg> = (msg: Msg) => void

/**
 * A side effect, described as data: the call `run(...args)`.
 *
 * The call always resolves to a follow-up message. `run` receives every input
 * as an argument. It reports an expected failure as a message and does not
 * throw.
 *
 * A command is data, so a test can compare the commands that `update` returns.
 */
export type Cmd<Msg> = {
  // `any` is the one parameter type that every function matches.
  readonly run: (...args: any) => Promise<Msg>
  readonly args: readonly unknown[]
}

/** Describes the command `run(...args)`. The kernel makes the call later. */
export const cmd = <Args extends readonly unknown[], const Msg>(
  run: (...args: Args) => Promise<Msg>,
  ...args: Args
): Cmd<Msg> => ({ run, args })

/**
 * A source of messages, described as data: the call `run(dispatch, ...args)`.
 *
 * `run` starts the source and returns a function that stops it. The source
 * calls `dispatch` for each message. After the stop, its messages are dropped.
 *
 * Two subscriptions are the same when they have the same `run` and the same
 * arguments, as compared by `Object.is`. Define `run` at module scope, and
 * pass it primitives or references that the state holds.
 */
export type Sub<Msg> = {
  readonly run: (dispatch: Dispatch<Msg>, ...args: any) => () => void
  readonly args: readonly unknown[]
}

/** Describes the subscription `run(dispatch, ...args)`. */
export const sub = <Args extends readonly unknown[], Msg>(
  run: (dispatch: Dispatch<Msg>, ...args: Args) => () => void,
  ...args: Args
): Sub<Msg> => ({ run, args })

/**
 * The outcome of one step: the next state, followed by the commands to launch.
 * `[state]` launches no command.
 */
export type Transition<State, Msg> = readonly [State, ...Cmd<Msg>[]]

/**
 * Computes the transition for one message. `update` is pure: it reads only its
 * arguments, causes no side effect, and does not throw.
 */
export type Update<State, Msg> = (
  state: State,
  msg: Msg,
) => Transition<State, NoInfer<Msg>>

/**
 * The description of an application. Defining a program runs nothing.
 *
 * The `msg` parameter of `update` is the one place that defines `Msg`. Every
 * place that produces a message is marked `NoInfer`. TypeScript checks those
 * places against `Msg` and never infers `Msg` from them.
 */
export type Program<State, Msg> = {
  /** The initial state, followed by the commands that `start` launches. */
  readonly init: Transition<State, NoInfer<Msg>>
  readonly update: Update<State, Msg>
  /**
   * Lists the subscriptions that a state needs. It is pure. After each state
   * change, the kernel stops the subscriptions that left the list and starts
   * the ones that entered it. A subscription that stays keeps running.
   */
  readonly subscriptions?: (state: State) => readonly Sub<NoInfer<Msg>>[]
}

/** A running program. It holds the state and processes the messages. */
export type Kernel<State, Msg> = {
  /** Returns the current state. */
  readonly getState: () => State
  /**
   * Registers `listener`. The kernel calls it after each state change. Returns
   * a function that removes the listener.
   */
  readonly subscribe: (listener: () => void) => () => void
  /**
   * Sends a message to `update`. The kernel processes one message at a time,
   * in order of arrival: it launches the commands, replaces the state,
   * notifies the listeners, then updates the subscriptions. A call made during
   * that work, for example by a listener, queues its message.
   *
   * `update` and `subscriptions` are pure. A call from either one throws.
   */
  readonly dispatch: Dispatch<Msg>
  /**
   * Registers `handler`. The kernel calls it with each defect: a value that
   * the program's code threw, whether in a command, a subscription, `update`,
   * or `subscriptions`. Returns a function that removes the handler.
   *
   * While no handler is registered, a defect is an unhandled promise
   * rejection, so the JavaScript runtime reports it.
   */
  readonly onDefect: (handler: (cause: unknown) => void) => () => void
  /**
   * Starts the kernel. The first call launches the commands of `init`. The
   * subscriptions run from the first `start` until every `start` is stopped.
   * Returns the function that stops this `start`.
   *
   * A stopped kernel keeps its state and still processes messages.
   */
  readonly start: () => () => void
}

/** Tells whether two descriptions denote the same subscription. */
const same = (a: Sub<unknown>, b: Sub<unknown>): boolean =>
  a.run === b.run
  && a.args.length === b.args.length
  && a.args.every((arg, at) => Object.is(arg, b.args[at]))

/** Adds an item to a set. Returns a function that removes the item. */
const register =
  <Item>(items: Set<Item>) =>
  (item: Item): (() => void) => {
    items.add(item)
    return () => void items.delete(item)
  }

/**
 * Creates a kernel that holds the initial state of the program.
 *
 * `createKernel` is pure. No command and no subscription runs before `start`.
 * React may therefore call it during a render and discard the result, as
 * StrictMode does.
 *
 * `Msg` is a `const` type parameter, so the messages of a program written
 * inline keep their literal types. Such a program needs no type arguments.
 */
export const createKernel = <State, const Msg>(
  program: Program<State, Msg>,
): Kernel<State, Msg> => {
  type Live = { readonly sub: Sub<Msg>; readonly close: () => void }

  const [initial, ...pending] = program.init
  let state = initial
  let live: readonly Live[] = []
  let computing = false
  const tasks: (() => void)[] = []
  const starts = new Set<object>()
  const listeners = new Set<() => void>()
  const handlers = new Set<(cause: unknown) => void>()

  // A defect is a rejected promise, and each handler receives its cause.
  // Without a handler the rejection stays unhandled, so it is never silent.
  const defect = (cause: unknown): void => {
    const rejection = Promise.reject(cause)
    handlers.forEach((handler) => rejection.catch(handler))
  }

  // Runs `work`. Whatever it throws is a defect.
  const attempt = (work: () => void): void => {
    try {
      work()
    } catch (cause) {
      defect(cause)
    }
  }

  // Runs `task` now, or after the running task when a task calls it. Tasks
  // therefore never interleave.
  const enqueue = (task: () => void): void => {
    tasks.push(task)
    // A longer queue means that a loop is running further up the stack. That
    // loop will reach this task.
    if (tasks.length > 1) return
    // The loop also visits the tasks that arrive while it runs.
    for (const next of tasks) attempt(next)
    tasks.length = 0
  }

  // Calls `update` or `subscriptions`. Both are pure, so `dispatch` throws
  // until the call returns.
  const purely = <Result>(compute: () => Result): Result => {
    computing = true
    try {
      return compute()
    } finally {
      computing = false
    }
  }

  // Makes the call in a microtask, so that no command code runs in a task.
  const launch = (cmd: Cmd<Msg>): void => {
    Promise.resolve()
      .then(() => cmd.run(...cmd.args))
      .then(dispatch, defect)
  }

  const open = (sub: Sub<Msg>): Live => {
    let on = true
    let stop = (): void => {}
    // The source dispatches through a gate that `close` shuts.
    const gate: Dispatch<Msg> = (msg) => {
      if (on) dispatch(msg)
    }
    attempt(() => {
      stop = sub.run(gate, ...sub.args)
    })
    return {
      sub,
      close: () => {
        on = false
        attempt(stop)
      },
    }
  }

  // Makes the live subscriptions match the ones that the state needs. It
  // closes the ones that left before it opens the ones that entered.
  const sync = (): void => {
    const needed =
      starts.size > 0 ? purely(() => program.subscriptions?.(state) ?? []) : []
    const idle = [...live]
    const kept = needed.map((sub) => {
      const at = idle.findIndex((entry) => same(entry.sub, sub))
      return at < 0 ? undefined : idle.splice(at, 1)[0]
    })
    idle.forEach((entry) => entry.close())
    live = needed.map((sub, at) => kept[at] ?? open(sub))
  }

  const dispatch = (msg: Msg): void => {
    if (computing) {
      throw new Error(
        "`update` and `subscriptions` are pure. They cannot dispatch.",
      )
    }
    enqueue(() => {
      const [next, ...cmds] = purely(() => program.update(state, msg))
      // The commands go first, so that no later failure can drop them.
      cmds.forEach(launch)
      if (next === state) return
      state = next
      listeners.forEach((listener) => listener())
      sync()
    })
  }

  return {
    getState: () => state,
    subscribe: register(listeners),
    dispatch,
    onDefect: register(handlers),
    start: () => {
      // The token names this call, so that its stop cannot end another one.
      const token = {}
      enqueue(() => {
        starts.add(token)
        // `splice` empties `pending`, so a later `start` launches nothing.
        pending.splice(0).forEach(launch)
        sync()
      })
      return () =>
        enqueue(() => {
          starts.delete(token)
          sync()
        })
    },
  }
}
