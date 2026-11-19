// プラグイン（Worker の中）からホストとメッセージをやり取りする薄いヘルパー。

import type { HostMessage, PluginMessage } from "./generated/protocol.js";

type MessageListener = (event: { data: unknown }) => void;

/** `postMessage` でやり取りする相手。Worker の中では `self` を渡す。 */
export interface MessageEndpoint {
  postMessage(message: unknown): void;
  addEventListener(type: "message", listener: MessageListener): void;
  removeEventListener(type: "message", listener: MessageListener): void;
}

export interface HostConnection {
  /** ホストからのメッセージを受け取る。返した関数を呼ぶと受け取りをやめる。 */
  onMessage(handler: (message: HostMessage) => void): () => void;
  /** ホストへメッセージを送る。 */
  send(message: PluginMessage): void;
}

// 中身の検証はホストを信じて行わない。`type` を持たないもの（他のライブラリのメッセージなど）だけを読み飛ばす。
function isTaggedMessage(data: unknown): data is { type: string } {
  return typeof data === "object" && data !== null && typeof (data as { type?: unknown }).type === "string";
}

/** ホストとの接続を作る。例: `const host = connectToHost(self);` */
export function connectToHost(endpoint: MessageEndpoint): HostConnection {
  return {
    onMessage(handler) {
      const listener: MessageListener = (event) => {
        if (isTaggedMessage(event.data)) handler(event.data as HostMessage);
      };
      endpoint.addEventListener("message", listener);
      return () => endpoint.removeEventListener("message", listener);
    },
    send(message) {
      endpoint.postMessage(message);
    },
  };
}
