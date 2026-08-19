import { Channel, invoke } from "@tauri-apps/api/core";
import {
  create,
  fromBinary,
  toBinary,
  type DescMessage,
  type DescMethodStreaming,
  type DescMethodUnary,
  type MessageInitShape,
} from "@bufbuild/protobuf";
import { Code, ConnectError, type Transport } from "@connectrpc/connect";

type GrpcFailure = { code: number; message: string };

type StreamEvent =
  | { kind: "message"; data: string }
  | { kind: "end" }
  | { kind: "error"; code: number; message: string };

function toConnectError(err: unknown): ConnectError {
  if (err !== null && typeof err === "object" && "code" in err) {
    const failure = err as GrpcFailure;
    return new ConnectError(failure.message, failure.code as Code);
  }
  return ConnectError.from(err);
}

function encodeBase64(bytes: Uint8Array): string {
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary);
}

function decodeBase64(value: string): Uint8Array {
  const binary = atob(value);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
  return bytes;
}

function methodPath(method: { parent: { typeName: string }; name: string }) {
  return `/${method.parent.typeName}/${method.name}`;
}

class EventQueue {
  private readonly pending: StreamEvent[] = [];
  private notify: (() => void) | null = null;

  push(event: StreamEvent) {
    this.pending.push(event);
    this.notify?.();
    this.notify = null;
  }

  async next(): Promise<StreamEvent> {
    const queued = this.pending.shift();
    if (queued) return queued;
    await new Promise<void>((resolve) => {
      this.notify = resolve;
    });
    return this.pending.shift()!;
  }
}

export function createTauriGrpcTransport(): Transport {
  return {
    async unary<I extends DescMessage, O extends DescMessage>(
      method: DescMethodUnary<I, O>,
      _signal: AbortSignal | undefined,
      _timeoutMs: number | undefined,
      _header: HeadersInit | undefined,
      input: MessageInitShape<I>,
    ) {
      const request = toBinary(method.input, create(method.input, input));
      try {
        const response = await invoke<string>("grpc_unary", {
          path: methodPath(method),
          request: encodeBase64(request),
        });
        return {
          stream: false as const,
          service: method.parent,
          method,
          message: fromBinary(method.output, decodeBase64(response)),
          header: new Headers(),
          trailer: new Headers(),
        };
      } catch (err: unknown) {
        throw toConnectError(err);
      }
    },

    async stream<I extends DescMessage, O extends DescMessage>(
      method: DescMethodStreaming<I, O>,
      signal: AbortSignal | undefined,
      _timeoutMs: number | undefined,
      _header: HeadersInit | undefined,
      input: AsyncIterable<MessageInitShape<I>>,
    ) {
      const first = await input[Symbol.asyncIterator]().next();
      if (first.done) {
        throw new ConnectError(
          `${methodPath(method)} requires a request message`,
          Code.Internal,
        );
      }

      const queue = new EventQueue();
      const channel = new Channel<StreamEvent>();
      channel.onmessage = (event) => queue.push(event);

      const request = toBinary(method.input, create(method.input, first.value));
      let streamId: number;
      try {
        streamId = await invoke<number>("grpc_server_stream", {
          path: methodPath(method),
          request: encodeBase64(request),
          onEvent: channel,
        });
      } catch (err: unknown) {
        throw toConnectError(err);
      }

      const cancel = () => void invoke("grpc_cancel_stream", { streamId });
      signal?.addEventListener("abort", cancel, { once: true });

      async function* messages() {
        try {
          for (;;) {
            const event = await queue.next();
            if (event.kind === "end") return;
            if (event.kind === "error") {
              throw new ConnectError(event.message, event.code as Code);
            }
            yield fromBinary(method.output, decodeBase64(event.data));
          }
        } finally {
          signal?.removeEventListener("abort", cancel);
          cancel();
        }
      }

      return {
        stream: true as const,
        service: method.parent,
        method,
        message: messages(),
        header: new Headers(),
        trailer: new Headers(),
      };
    },
  };
}
