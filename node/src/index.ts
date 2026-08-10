const native: {
  pidOrder(): string[];
  canonicalString(value: string): string;
  canonicalNationality(value: string): string;
  canonicalNationalityList(values: string[]): string;
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

export function canonicalString(value: string): string {
  return native.canonicalString(value);
}

export function canonicalNationality(value: string): string {
  return native.canonicalNationality(value);
}

export function canonicalNationalityList(values: readonly string[]): string {
  return native.canonicalNationalityList([...values]);
}

export function verifyProof(
  publicKey: Uint8Array,
  proof: Uint8Array,
  revealedMessages: readonly RevealedMessage[],
): boolean {
  return native.verifyProof(publicKey, proof, [...revealedMessages]);
}
