import { describe, expect, it } from "vitest";
import { connectToHost, type MessageEndpoint } from "./channel.js";
import type { HostMessage } from "./index.js";

type Listener = (event: { data: unknown }) => void;

/** Worker の `self` の代わり。送ったものを記録し、`deliver` で受信を起こす。 */
function fakeEndpoint() {
  const listeners = new Set<Listener>();
  const posted: unknown[] = [];
  const endpoint: MessageEndpoint = {
    postMessage: (message) => posted.push(message),
    addEventListener: (_, listener) => listeners.add(listener),
    removeEventListener: (_, listener) => listeners.delete(listener),
  };
  const deliver = (data: unknown) => listeners.forEach((l) => l({ data }));
  return { endpoint, posted, deliver };
}

describe("connectToHost", () => {
  it("passes host messages to the handler", () => {
    // Arrange
    const { endpoint, deliver } = fakeEndpoint();
    const received: HostMessage[] = [];
    connectToHost(endpoint).onMessage((m) => received.push(m));
    const message: HostMessage = { type: "runCommand", requestId: 1, command: "fmea.open" };
    // Act
    deliver(message);
    // Assert
    expect(received).toEqual([message]);
  });

  it.each([null, "ready", 1, {}, { type: 1 }])("ignores data without a string type: %j", (data: unknown) => {
    // Arrange（異常系: 他のライブラリのメッセージなど）
    const { endpoint, deliver } = fakeEndpoint();
    const received: HostMessage[] = [];
    connectToHost(endpoint).onMessage((m) => received.push(m));
    // Act
    deliver(data);
    // Assert
    expect(received).toEqual([]);
  });

  it("stops receiving after unsubscribe", () => {
    // Arrange
    const { endpoint, deliver } = fakeEndpoint();
    const received: HostMessage[] = [];
    const stop = connectToHost(endpoint).onMessage((m) => received.push(m));
    // Act
    stop();
    deliver({ type: "runCommand", requestId: 1, command: "x" });
    // Assert
    expect(received).toEqual([]);
  });

  it("posts plugin messages to the host", () => {
    // Arrange
    const { endpoint, posted } = fakeEndpoint();
    const host = connectToHost(endpoint);
    // Act
    host.send({ type: "ready" });
    // Assert
    expect(posted).toEqual([{ type: "ready" }]);
  });
});
