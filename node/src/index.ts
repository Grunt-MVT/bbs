const native: {
  pidOrder(): string[];
  verifyProof(
    publicKey: Uint8Array,
    proof: Uint8Array,
    revealedMessages: RevealedMessage[],
  ): boolean;
} = require("../native/bbsplus_node.node");

export type RevealedMessage = {
  index: number;
  data: Uint8Array;
};

export const maxMessageCount = 20;

export const pidOrder: readonly string[] = Object.freeze([...native.pidOrder()]);

export function verifyProof(
  publicKey: Uint8Array,
  proof: Uint8Array,
  revealedMessages: readonly RevealedMessage[],
): boolean {
  return native.verifyProof(publicKey, proof, [...revealedMessages]);
}
