# Specification: The Elm Architecture for React

`@mayflower-sys/react-elm` is a small runtime for The Elm Architecture in React. Its only dependency is `react`. An application that uses it needs no third-party state library.

The package has two modules:

- `src/kernel.ts` is the runtime. It imports nothing, so it also runs outside React.
- `src/react.ts` connects a kernel to React components through `useSyncExternalStore`.

---

## 1. Concepts

```
    ┌────────────────────────────────────────────────────────┐
    │                       React View                       │
    │  - Reads state through useSelect(select)               │
    │  - Sends messages through useDispatch()                │
    └──────────────────┬─────────────────▲───────────────────┘
                       │                 │
                 dispatch(msg)       re-render
                       │                 │
    ┌──────────────────▼─────────────────┴───────────────────┐
    │                         Kernel                         │
    │  - Processes one message at a time, in arrival order   │
    │  - Notifies React through useSyncExternalStore         │
    │  - Passes each defect to the onDefect handlers         │
    └──────────────────┬─────────────────▲───────────────────┘
                       │                 │
                  state, msg       dispatch(msg)
                       │                 │
    ┌──────────────────▼────────┐        │
    │       Pure functions      │        │
    │  update(state, msg)       │        │
    │    -> [State, ...Cmd[]]   │        │
    │  subscriptions(state)     │        │
    │    -> Sub[]               │        │
    └──────────────┬────────────┘        │
                   │                     │
            Cmd[] and Sub[]              │
                   │                     │
    ┌──────────────▼─────────────────────┴───────────────────┐
    │                     Effect runner                      │
    │  - Makes each command's call, and does not wait for it │
    │  - Keeps each listed subscription open                 │
    │  - Effects cannot read the state                       │
    │  - A throw is a defect                                 │
    └────────────────────────────────────────────────────────┘
```

1. **State.** Immutable data. It holds everything that the view shows.
2. **Message (`Msg`).** A discriminated union. A message records an event that has already happened.
3. **Update.** A pure function `(state, msg) => [nextState, ...cmds]`. It does not mutate the state, and it performs no side effect.
4. **Command (`Cmd<Msg>`).** A side effect, described as data: a function and the arguments to call it with. `cmd(fetchBlock, height, id)` describes the call `fetchBlock(height, id)`. The call always resolves to a follow-up message.
5. **Subscription (`Sub<Msg>`).** A source of messages, described as data in the same way. `subscriptions(state)` lists the sources that a state needs. The kernel opens and closes sources so that the open ones always match the list.
6. **Defect.** A value that the program's code threw. An expected failure is not a defect: a command reports it as a message. The kernel passes each defect to the handlers that the application registered.
7. **Kernel.** The runtime. It holds the state, processes one message at a time, and runs the commands and the subscriptions.

Commands and subscriptions are data. `update` and `subscriptions` are therefore pure functions whose whole result a test can compare.

---

## 2. Kernel (`src/kernel.ts`)

```ts
type Dispatch<Msg> = (msg: Msg) => void

// A side effect as data: the call `run(...args)`.
type Cmd<Msg> = {
  readonly run: (...args: any) => Promise<Msg>
  readonly args: readonly unknown[]
}

// A source of messages as data: the call `run(dispatch, ...args)`. The call
// starts the source and returns the function that stops it.
type Sub<Msg> = {
  readonly run: (dispatch: Dispatch<Msg>, ...args: any) => () => void
  readonly args: readonly unknown[]
}

declare const cmd: <Args extends readonly unknown[], Msg>(
  run: (...args: Args) => Promise<Msg>,
  ...args: Args
) => Cmd<Msg>

declare const sub: <Args extends readonly unknown[], Msg>(
  run: (dispatch: Dispatch<Msg>, ...args: Args) => () => void,
  ...args: Args
) => Sub<Msg>

// The next state, followed by the commands to launch. `[state]` launches none.
type Transition<State, Msg> = readonly [State, ...Cmd<Msg>[]]

type Update<State, Msg> = (state: State, msg: Msg) => Transition<State, Msg>

type Program<State, Msg> = {
  readonly init: Transition<State, Msg>
  readonly update: Update<State, Msg>
  readonly subscriptions?: (state: State) => readonly Sub<Msg>[]
}

type Kernel<State, Msg> = {
  readonly getState: () => State
  readonly subscribe: (listener: () => void) => () => void
  readonly dispatch: Dispatch<Msg>
  readonly onDefect: (handler: (cause: unknown) => void) => () => void
  readonly start: () => () => void
}

declare const createKernel: <State, Msg>(
  program: Program<State, Msg>,
) => Kernel<State, Msg>
```

The kernel keeps these guarantees:

1. **Creation is pure.** `createKernel` launches no command and opens no subscription.
2. **`start` runs the effects.** The first `start` launches the commands of `init`. No later `start` launches them again. The subscriptions run from the first `start` until every `start` is stopped. `start` returns the function that stops it. A stopped kernel keeps its state and still processes messages, so a command in flight still delivers its message.
3. **One message at a time.** `dispatch` processes messages in order of arrival. For each message, the kernel calls `update` and launches the commands. If the new state is not (`!==`) the old one, the kernel then replaces the state, notifies the listeners, and updates the subscriptions. A `dispatch` call made during that work queues its message behind the current one.
4. **The pure functions cannot dispatch.** A `dispatch` call made while `update` or `subscriptions` runs throws, and the throw is a defect.
5. **Commands run outside `dispatch`.** The kernel makes each call in a microtask and does not wait for it. It dispatches the message that the call resolves to. Commands run concurrently, so their messages arrive in order of completion.
6. **Subscriptions follow the state.** After each state change, the kernel calls `subscriptions(state)` and compares the list with the open subscriptions. It closes the ones that left the list, then opens the ones that entered it. A subscription that is still listed keeps running. Two descriptions denote the same subscription when they have the same `run` and the same arguments, as compared by `Object.is`. After the kernel closes a subscription, it drops every message that the source still sends.
7. **A defect is never silent.** A defect is a value that the program's code threw: in a command, in a subscription, in `update`, or in `subscriptions`. The kernel passes each defect to every handler registered with `onDefect`. While no handler is registered, the defect is an unhandled promise rejection, which the JavaScript runtime reports. The kernel stays usable after a defect.

The source refines these types for inference. The refinements do not change what the types accept. `createKernel` infers `Msg` from one place only: the `msg` parameter of `update`. TypeScript checks the messages that `init`, the commands, and the subscriptions produce against that type. A program written inline therefore needs no type arguments.

---

## 3. React bindings (`src/react.ts`)

```ts
const { Provider, useSelect, useDispatch } = createElmContext<State, Msg>()
```

Call `createElmContext` once for each `State` and `Msg` pair, at module scope.

- **`<Provider kernel={kernel}>`** gives the kernel to the hooks of the components below. It starts the kernel when it mounts and stops it when it unmounts.
- **`useSelect(select)`** returns `select(state)`. It re-renders the component only when that value changes, as compared by `Object.is`.
- **`useDispatch()`** returns the kernel's `dispatch`. Its identity never changes, so it never causes a re-render.

The bindings do not expose defects. The code that creates the kernel registers the handlers with `kernel.onDefect`. The bindings do not expose the kernel either: a component can select and dispatch, and nothing else.

### Development mode

In development, React's StrictMode renders each component twice and runs each effect twice. The package tolerates both:

- `createKernel` is pure. A kernel that React creates during a render and then discards has run nothing. `useState(() => createKernel(program))` is therefore safe.
- The Provider's effect starts the kernel, stops it, and starts it again. The commands of `init` launch once. Each subscription opens, closes, and opens again, as the effect of any React component does.
- `useSyncExternalStore` subscribes, unsubscribes, and subscribes again. `subscribe` only adds and removes a listener.

Create the kernel where its lifetime belongs: at module scope to live as long as the page, or in `useState` to live as long as a component. When the Provider unmounts, the kernel closes its subscriptions. A command in flight still completes. Describe a repeating source as a subscription, not as a command that launches itself again: only a subscription stops with the kernel.

Call `createElmContext` in a module that exports no component. React Fast Refresh updates a module in place only when every export is a component. When a component module also exports the bindings, Vite reloads the page on each edit of that module, and the kernel loses its state.

---

## 4. Example: the program (`explorer.ts`)

A block explorer. The example shows:

- A request id that drops stale responses.
- A command that receives every input as an argument.
- An expected failure, which is a message, and a defect, which is a throw.
- A subscription that runs exactly while the state asks for it.
- A state type that cannot hold a loaded view without a block.

```ts
import {
  cmd,
  sub,
  type Dispatch,
  type Program,
  type Update,
} from "@mayflower-sys/react-elm"

// 1. State

export type Block = {
  readonly height: number
  readonly hash: string
  readonly txCount: number
}

export type State = {
  /** The id of the latest request. A response that carries another id is stale. */
  readonly requestId: number
  /** Whether the explorer shows each new block as the chain produces it. */
  readonly following: boolean
  readonly view:
    | { readonly status: "idle" }
    | { readonly status: "loading" }
    | { readonly status: "loaded"; readonly block: Block }
    | { readonly status: "failed"; readonly reason: string }
}

// 2. Messages

export type Msg =
  | { type: "QUERY_BLOCK_REQUESTED"; height: number }
  | { type: "BLOCK_FETCH_SUCCEEDED"; requestId: number; block: Block }
  | { type: "BLOCK_FETCH_FAILED"; requestId: number; reason: string }
  | { type: "VIEW_CLEARED" }
  | { type: "FOLLOW_TOGGLED" }
  | { type: "TIP_ADVANCED"; height: number }
  | { type: "DEFECT_REPORTED"; cause: unknown }

// 3. Commands

/** Fetches one block. A command is an ordinary async function. */
export const fetchBlock = async (
  height: number,
  requestId: number,
): Promise<Msg> => {
  // Stands in for the RPC round trip.
  await new Promise((resolve) => setTimeout(resolve, 400))

  // An expected failure is a message.
  if (height < 0) {
    return {
      type: "BLOCK_FETCH_FAILED",
      requestId,
      reason: `Block ${height} does not exist.`,
    }
  }

  // A bug throws. The kernel reports it to the defect handlers.
  if (height === 666) throw new Error("The RPC client crashed.")

  return {
    type: "BLOCK_FETCH_SUCCEEDED",
    requestId,
    block: {
      height,
      hash: `0x${height.toString(16).padStart(64, "0")}`,
      txCount: height % 500,
    },
  }
}

// 4. Subscriptions

/** Reports each new chain tip. Returns the function that stops the feed. */
export const chainTip = (dispatch: Dispatch<Msg>, intervalMs: number) => {
  // Stands in for a WebSocket subscription.
  let height = 1_000
  const timer = setInterval(() => {
    height += 1
    dispatch({ type: "TIP_ADVANCED", height })
  }, intervalMs)
  return () => clearInterval(timer)
}

// 5. Update

export const update: Update<State, Msg> = (state, msg) => {
  switch (msg.type) {
    case "QUERY_BLOCK_REQUESTED": {
      const requestId = state.requestId + 1
      return [
        { ...state, requestId, view: { status: "loading" } },
        cmd(fetchBlock, msg.height, requestId),
      ]
    }

    // The view keeps the current block until the next one arrives.
    case "TIP_ADVANCED": {
      const requestId = state.requestId + 1
      return [{ ...state, requestId }, cmd(fetchBlock, msg.height, requestId)]
    }

    case "BLOCK_FETCH_SUCCEEDED":
      return msg.requestId === state.requestId ?
          [{ ...state, view: { status: "loaded", block: msg.block } }]
        : [state]

    case "BLOCK_FETCH_FAILED":
      return msg.requestId === state.requestId ?
          [{ ...state, view: { status: "failed", reason: msg.reason } }]
        : [state]

    // A new id retires the request in flight, so its response is dropped.
    case "VIEW_CLEARED":
      return [
        { ...state, requestId: state.requestId + 1, view: { status: "idle" } },
      ]

    case "FOLLOW_TOGGLED":
      return [{ ...state, following: !state.following }]

    case "DEFECT_REPORTED":
      return [
        {
          ...state,
          following: false,
          view: { status: "failed", reason: String(msg.cause) },
        },
      ]
  }
}

// 6. Program

export const program: Program<State, Msg> = {
  init: [{ requestId: 0, following: false, view: { status: "idle" } }],
  update,
  // The feed runs exactly while `following` is true.
  subscriptions: (state) => (state.following ? [sub(chainTip, 2_000)] : []),
}
```

The `switch` has no `default` branch. When `Msg` gains a variant, `update` no longer returns on every path, and the compiler rejects it.

---

## 5. Example: the view (`bindings.ts`, `App.tsx`, `main.tsx`)

```ts
// bindings.ts
import { createElmContext } from "@mayflower-sys/react-elm"
import type { Msg, State } from "./explorer.js"

export const { Provider, useSelect, useDispatch } = createElmContext<
  State,
  Msg
>()
```

```tsx
// App.tsx
import { useState } from "react"
import { useDispatch, useSelect } from "./bindings.js"

// Re-renders only when the `loading` boolean flips.
const SearchBar = () => {
  const [height, setHeight] = useState("100")
  const dispatch = useDispatch()
  const loading = useSelect((state) => state.view.status === "loading")

  return (
    <form
      onSubmit={(event) => {
        event.preventDefault()
        dispatch({ type: "QUERY_BLOCK_REQUESTED", height: Number(height) })
      }}
    >
      <input
        type="number"
        value={height}
        disabled={loading}
        onChange={(event) => setHeight(event.target.value)}
      />
      <button type="submit" disabled={loading}>
        {loading ? "Querying…" : "Fetch block"}
      </button>
      <button type="button" onClick={() => dispatch({ type: "VIEW_CLEARED" })}>
        Clear
      </button>
    </form>
  )
}

const FollowToggle = () => {
  const dispatch = useDispatch()
  const following = useSelect((state) => state.following)

  return (
    <label>
      <input
        type="checkbox"
        checked={following}
        onChange={() => dispatch({ type: "FOLLOW_TOGGLED" })}
      />
      Follow the chain tip
    </label>
  )
}

// `update` replaces `state.view` only when the view changes, so the selector
// returns a stable object.
const BlockViewer = () => {
  const view = useSelect((state) => state.view)

  switch (view.status) {
    case "idle":
      return <p>Enter a block height.</p>
    case "loading":
      return <p>Fetching the block…</p>
    case "failed":
      return <p role="alert">{view.reason}</p>
    case "loaded":
      return (
        <dl>
          <dt>Height</dt>
          <dd>{view.block.height}</dd>
          <dt>Hash</dt>
          <dd>{view.block.hash}</dd>
          <dt>Transactions</dt>
          <dd>{view.block.txCount}</dd>
        </dl>
      )
  }
}

export const App = () => (
  <main>
    <h1>Block explorer</h1>
    <SearchBar />
    <FollowToggle />
    <BlockViewer />
  </main>
)
```

```tsx
// main.tsx
import { StrictMode } from "react"
import { createRoot } from "react-dom/client"
import { createKernel } from "@mayflower-sys/react-elm"
import { App } from "./App.js"
import { Provider } from "./bindings.js"
import { program } from "./explorer.js"

const kernel = createKernel(program)

// The application decides what a defect means. This one shows it in the view.
kernel.onDefect((cause) => kernel.dispatch({ type: "DEFECT_REPORTED", cause }))

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <Provider kernel={kernel}>
      <App />
    </Provider>
  </StrictMode>,
)
```

---

## 6. Example: a test (`explorer.test.ts`)

A transition is data, and so is a list of subscriptions. One comparison covers the next state and the effects.

```ts
import assert from "node:assert/strict"
import { test } from "node:test"
import { cmd, sub } from "@mayflower-sys/react-elm"
import { chainTip, fetchBlock, program, update } from "./explorer.js"

const [initial] = program.init

test("a query starts loading and fetches the block under a new id", () => {
  assert.deepEqual(
    update(initial, { type: "QUERY_BLOCK_REQUESTED", height: 100 }),
    [
      { requestId: 1, following: false, view: { status: "loading" } },
      cmd(fetchBlock, 100, 1),
    ],
  )
})

test("the chain tip feed runs only while following", () => {
  assert.deepEqual(program.subscriptions?.(initial), [])
  assert.deepEqual(program.subscriptions?.({ ...initial, following: true }), [
    sub(chainTip, 2_000),
  ])
})
```

---

## 7. Rules

The kernel cannot enforce these rules. Code review must.

### Rule 1: An effect receives every input as an argument

The function of a command or of a subscription does not call `getState`, and it does not read a global variable. `update` and `subscriptions` read the state and pass each value.

An effect does not call the kernel's `dispatch` either. A command returns its one message. A subscription uses the `dispatch` that the kernel passes to it. The kernel cannot detect a command that breaks this rule: the command runs in a later turn of the event loop, where its call looks like any other. So keep the kernel out of module exports. Then no effect and no `update` can reach it.

```ts
// Wrong: the command reads the state.
const fetchUser = async (): Promise<Msg> => {
  const token = kernel.getState().authToken
  // ...
}

// Right: `update` reads the state and passes the token.
const fetchUser = async (token: string): Promise<Msg> => {
  // ...
}
const update: Update<State, Msg> = (state) => [
  state,
  cmd(fetchUser, state.authToken),
]
```

### Rule 2: `update` and `subscriptions` are pure

They do not call `localStorage`, `Math.random()`, `Date.now()`, or a logger. When a transition needs a timestamp or a random value, a command produces the value and returns it in a message.

They do not call `dispatch`. The kernel enforces this part of the rule: the call throws.

### Rule 3: An expected failure is a message, and a defect is a bug

A command catches the failures that it can expect, such as an HTTP error status or a rejected wallet signature. It returns each one as a message and does not throw it.

Whatever the program's code throws is a defect. The application decides what a defect means. Its `onDefect` handler can log the cause, report it, or dispatch a message, as the example does. If the handler dispatches a message, the branch of `update` that receives it launches no command that can throw. Otherwise each defect causes the next one, without end.

### Rule 4: A request carries an id

When responses can arrive out of order, `update` stores a new id in the state for each request and passes that id to the command. The command returns the id in its message. `update` drops a message whose id is not the stored one.

### Rule 5: A subscription keeps its identity

The kernel restarts a subscription when its `run` or one of its arguments is not (`Object.is`) the previous one.

- Define `run` at module scope. A function created inside `subscriptions` is new on each call, so the kernel would restart the source after every state change.
- Pass primitives, or references that the state holds. An object or an array built inside `subscriptions` is new on each call.
- `run` builds its own messages. Do not pass it a callback that builds them.

### Rule 6: A selector returns a small, stable value

`useSelect` passes `() => select(kernel.getState())` to `useSyncExternalStore`. React calls that function more than once for each render and compares the results with `Object.is`. A different result means that the state changed, so React renders again. `select` must therefore return the identical value each time it receives the same state.

- Select a field, or compute a primitive: `(state) => state.view`, `(state) => state.view.status === "loading"`.
- Do not build an object or an array in a selector. Each call returns a new one, and React aborts the render with "Maximum update depth exceeded". `useCallback` does not help: it keeps the function stable, not its result.
- To combine or derive values, call `useSelect` once for each raw value and merge the results with `useMemo`. The merge then runs only when one of the raw values changes:

  ```tsx
  const items = useSelect((state) => state.items)
  const filter = useSelect((state) => state.filter)
  const visible = useMemo(
    () => items.filter((item) => item.kind === filter),
    [items, filter],
  )
  ```

- Do not select the whole state. The component would re-render on every message.

### Rule 7: A change is a new reference

The kernel and `useSelect` detect a change by reference only. The kernel notifies its listeners when `update` returns a state that is not (`!==`) the previous one. `useSelect` re-renders its component when the selected value is not (`Object.is`) the previous one. Neither compares contents. `update` therefore keeps every reference that did not change:

- Return `[state]` when the message changes nothing.
- Do not mutate. Copy the objects on the path to the change, and reuse every other branch.

For a nested write, an application can use `optics-ts`. It copies the path and reuses the rest. It is a dependency of the application, not of this package.

```ts
import * as O from "optics-ts"

type View = { readonly status: "idle" } | { readonly status: "loading" }
type State = {
  readonly wallet: { readonly address: string; readonly lamports: number }
  readonly view: View
}

const lamports = O.optic<State>().prop("wallet").prop("lamports")
const view = O.optic<State>().prop("view")

// Copies `state` and `state.wallet`. Reuses `state.view`.
const credited = O.set(lamports)(5_000)(state)

// Returns `state` itself when the balance did not change.
const synced =
  msg.lamports === state.wallet.lamports ?
    state
  : O.set(lamports)(msg.lamports)(state)

// Names the union, so that "loading" keeps its literal type.
const refreshing = O.set(view)<View>({ status: "loading" })(state)
```

Three properties of `optics-ts` 2.4.1 shape this code:

- A write copies the path even when the new value equals the old one. Under reference equality that is a change. Compare first where a message can repeat a value.
- A literal written into a union or a literal type is widened, and the result has the type `DisallowedTypeChange`. Name the type as above, or write the value `as const`.
- `find`, `elems`, `at`, and `filter` do not accept a `readonly T[]`. Declare the array as `T[]`, or key the collection by id in a `Readonly<Record<string, T>>`.
