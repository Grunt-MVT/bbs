import path from "node:path";

type NativeBinding = {
  pidOrder(): string[];
  canonicalString(value: string): string;
  canonicalNationality(value: string): string;
  canonicalNationalityList(values: string[]): string;
  verifyProof(
    publicKey: Uint8Array,
    proof: Uint8Array,
    revealedMessages: RevealedMessage[],
  ): boolean;
};

const platformDirByTarget: Record<string, string> = {
  "darwin-arm64": "darwin_arm64",
  "linux-x64": "linux_amd64",
};

function resolveNativeAddonPath(): string {
  const platformKey = `${process.platform}-${process.arch}`;
  const platformDir = platformDirByTarget[platformKey];
  if (!platformDir) {
    throw new Error(
      `libbbsplus: bundled Node native addon is available only on darwin/arm64 or linux/amd64 (got ${platformKey})`,
    );
  }

  return path.join(__dirname, "..", "native", platformDir, "bbsplus_node.node");
}

const native: NativeBinding = require(resolveNativeAddonPath());

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

/** @internal Exported for tests. */
export function nativeAddonPathForTest(): string {
  return resolveNativeAddonPath();
}
