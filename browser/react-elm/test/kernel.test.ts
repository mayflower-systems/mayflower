import assert from "node:assert/strict"
import { once } from "node:events"
import { test } from "node:test"
import {
  cmd,
  createKernel,
  sub,
  type Cmd,
  type Dispatch,
  type Kernel,
  type Program,
} from "../src/kernel.ts"

type Log = readonly string[]

/**
 * A program that logs every message it receives. `cmds` maps a message to the
 * commands that `update` launches for it.
 */
const logger = (
  cmds: Readonly<Record<string, readonly Cmd<string>[]>> = {},
  ...init: readonly Cmd<string>[]
): Program<Log, string> => ({
  init: [[], ...init],
  update: (log, msg) => [[...log, msg], ...(cmds[msg] ?? [])],
})

/** A program that sums the numbers it receives. Its `update` throws on NaN. */
const adder: Program<number, number> = {
  init: [0],
  update: (sum, n) => {
    if (Number.isNaN(n)) throw new Error("not a number")
    return [sum + n]
  },
}

/** Resolves once every command that awaits only promises has finished. */
const settled = (): Promise<void> =>
  new Promise((resolve) => setImmediate(resolve))

/**
 * Resolves with the cause of the next unhandled rejection. The test runner
 * fails a test for an unhandled rejection, so its listeners step aside.
 */
const nextUnhandledRejection = async (): Promise<unknown> => {
  const runner = process.listeners("unhandledRejection")
  process.removeAllListeners("unhandledRejection")
  try {
    const [cause] = await once(process, "unhandledRejection")
    return cause
  } finally {
    runner.forEach((listener) => process.on("unhandledRejection", listener))
  }
}

test("only the first start launches the init commands", async () => {
  const kernel = createKernel(
    logger(
      {},
      cmd(async () => "ready"),
    ),
  )
  await settled()
  assert.deepEqual(kernel.getState(), [])

  const stop = kernel.start()
  stop()
  kernel.start()
  await settled()
  assert.deepEqual(kernel.getState(), ["ready"])
})

test("commands are data until the kernel runs them", async () => {
  let ran = false
  const greet = async (greeting: string, name: string) => {
    ran = true
    return `${greeting}, ${name}`
  }
  const program = logger({ go: [cmd(greet, "hello", "kernel")] })
  assert.deepEqual(program.update([], "go"), [
    ["go"],
    cmd(greet, "hello", "kernel"),
  ])

  const kernel = createKernel(program)
  kernel.dispatch("go")
  assert.equal(ran, false)
  await settled()
  assert.deepEqual(kernel.getState(), ["go", "hello, kernel"])
})

test("what a command throws reaches every defect handler", async () => {
  const thrown = new Error("thrown")
  const rejected = new Error("rejected")
  const kernel = createKernel(
    logger({
      sync: [
        cmd((): Promise<string> => {
          throw thrown
        }),
      ],
      async: [cmd(() => Promise.reject(rejected))],
    }),
  )
  const first: unknown[] = []
  const second: unknown[] = []
  kernel.onDefect((cause) => first.push(cause))
  kernel.onDefect((cause) => second.push(cause))

  kernel.dispatch("sync")
  await settled()
  kernel.dispatch("async")
  await settled()
  assert.deepEqual(first, [thrown, rejected])
  assert.deepEqual(second, [thrown, rejected])
  assert.deepEqual(kernel.getState(), ["sync", "async"])
})

test("an update that throws is a defect, and the kernel goes on", async () => {
  const kernel = createKernel(adder)
  const defects: unknown[] = []
  kernel.onDefect((cause) => defects.push(String(cause)))

  kernel.dispatch(NaN)
  kernel.dispatch(2)
  await settled()
  assert.deepEqual(defects, ["Error: not a number"])
  assert.equal(kernel.getState(), 2)
})

test(
  "without a handler, a defect is an unhandled rejection",
  {
    timeout: 1_000,
  },
  async () => {
    const kernel = createKernel(adder)
    const remove = kernel.onDefect(() => assert.fail("the handler was removed"))
    remove()

    const unhandled = nextUnhandledRejection()
    kernel.dispatch(NaN)
    assert.equal(String(await unhandled), "Error: not a number")
  },
)

test("a listener that throws does not drop the commands", async () => {
  const kernel = createKernel(logger({ go: [cmd(async () => "done")] }))
  const defects: unknown[] = []
  kernel.onDefect((cause) => defects.push(String(cause)))
  kernel.subscribe(() => {
    throw new Error("listener failed")
  })

  kernel.dispatch("go")
  await settled()
  assert.deepEqual(kernel.getState(), ["go", "done"])
  assert.deepEqual(defects, Array(2).fill("Error: listener failed"))
})

test("update and subscriptions cannot dispatch", async () => {
  const kernel: Kernel<Log, string> = createKernel<Log, string>({
    init: [[]],
    update: (log, msg) => {
      if (msg === "from update") kernel.dispatch("smuggled")
      return [[...log, msg]]
    },
    subscriptions: (log) => {
      if (log.at(-1) === "from subscriptions") kernel.dispatch("smuggled")
      return []
    },
  })
  const defects: unknown[] = []
  kernel.onDefect((cause) => defects.push(cause))
  kernel.start()

  kernel.dispatch("from update")
  kernel.dispatch("from subscriptions")
  kernel.dispatch("after")
  await settled()
  assert.deepEqual(kernel.getState(), ["from subscriptions", "after"])
  assert.equal(defects.length, 2)
  defects.forEach((cause) => assert.match(String(cause), /cannot dispatch/))
})

test("listeners hear state changes only, until they unsubscribe", () => {
  const kernel = createKernel(adder)
  let calls = 0
  const unsubscribe = kernel.subscribe(() => calls++)

  kernel.dispatch(1)
  kernel.dispatch(0)
  unsubscribe()
  kernel.dispatch(1)
  assert.equal(calls, 1)
  assert.equal(kernel.getState(), 2)
})

test("a dispatch from a listener waits for the current message", () => {
  const kernel = createKernel(logger())
  const seen: Log[] = []
  kernel.subscribe(() => {
    if (kernel.getState().length === 1) kernel.dispatch("second")
  })
  kernel.subscribe(() => seen.push(kernel.getState()))

  kernel.dispatch("first")
  assert.deepEqual(seen, [["first"], ["first", "second"]])
})

type Radio = {
  readonly channels: readonly string[]
  readonly heard: readonly string[]
}
type RadioMsg =
  | { readonly type: "tuned"; readonly channels: readonly string[] }
  | { readonly type: "heard"; readonly channel: string }

test("subscriptions follow the state while the kernel is started", () => {
  const journal: string[] = []
  const speakers = new Map<string, Dispatch<RadioMsg>>()
  const listen = (dispatch: Dispatch<RadioMsg>, channel: string) => {
    journal.push(`open ${channel}`)
    speakers.set(channel, dispatch)
    return () => void journal.push(`close ${channel}`)
  }
  const speak = (channel: string): void =>
    speakers.get(channel)?.({ type: "heard", channel })
  const kernel = createKernel<Radio, RadioMsg>({
    init: [{ channels: [], heard: [] }],
    update: (radio, msg) =>
      msg.type === "tuned" ?
        [{ ...radio, channels: msg.channels }]
      : [{ ...radio, heard: [...radio.heard, msg.channel] }],
    subscriptions: (radio) =>
      radio.channels.map((channel) => sub(listen, channel)),
  })

  // No subscription opens before `start`.
  kernel.dispatch({ type: "tuned", channels: ["a", "b"] })
  assert.deepEqual(journal, [])
  const stop = kernel.start()
  assert.deepEqual(journal.splice(0), ["open a", "open b"])

  // A state change that still lists a subscription does not restart it.
  speak("a")
  assert.deepEqual(kernel.getState().heard, ["a"])
  assert.deepEqual(journal.splice(0), [])

  // The kernel closes what left the list before it opens what entered.
  kernel.dispatch({ type: "tuned", channels: ["b", "c"] })
  assert.deepEqual(journal.splice(0), ["close a", "open c"])

  // A closed subscription can no longer dispatch.
  speak("a")
  assert.deepEqual(kernel.getState().heard, ["a"])

  // The subscriptions run until every `start` is stopped.
  const stopOther = kernel.start()
  stop()
  stop()
  assert.deepEqual(journal.splice(0), [])
  stopOther()
  assert.deepEqual(journal.splice(0), ["close b", "close c"])
})

test("a failing subscription is a defect, and the rest still run", async () => {
  const journal: string[] = []
  const failsToOpen = (_dispatch: Dispatch<string>): (() => void) => {
    throw new Error("open failed")
  }
  const failsToClose = (_dispatch: Dispatch<string>) => () => {
    throw new Error("close failed")
  }
  const healthy = (_dispatch: Dispatch<string>) => {
    journal.push("open")
    return () => void journal.push("close")
  }
  const kernel = createKernel({
    ...logger(),
    subscriptions: () => [sub(failsToOpen), sub(failsToClose), sub(healthy)],
  })
  const defects: unknown[] = []
  kernel.onDefect((cause) => defects.push(String(cause)))

  const stop = kernel.start()
  stop()
  await settled()
  assert.deepEqual(defects, ["Error: open failed", "Error: close failed"])
  assert.deepEqual(journal, ["open", "close"])
})
