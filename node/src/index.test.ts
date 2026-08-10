import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";

import { pidOrder, verifyProof, type RevealedMessage } from "./index";

type Fixture = {
  publicKey: string;
  proof: string;
  revealedMessages: Array<{
    index: number;
    data: string;
  }>;
};

function bytes(base64: string): Uint8Array {
  return Buffer.from(base64, "base64");
}

function loadFixture() {
  const fixturePath = path.join(__dirname, "..", "test", "fixtures", "proof.json");
  const fixture = JSON.parse(fs.readFileSync(fixturePath, "utf8")) as Fixture;

  return {
    publicKey: bytes(fixture.publicKey),
    proof: bytes(fixture.proof),
    revealedMessages: fixture.revealedMessages.map<RevealedMessage>((message) => ({
      index: message.index,
      data: bytes(message.data),
    })),
  };
}

test("pidOrder exposes protocol PID identifiers", () => {
  assert.deepEqual(pidOrder, [
    "family_name",
    "given_name",
    "birth_date",
    "birth_place",
    "nationality",
    "derived_nationality",
  ]);
});

test("verifyProof returns true for a valid proof", () => {
  const fixture = loadFixture();

  assert.equal(
    verifyProof(fixture.publicKey, fixture.proof, fixture.revealedMessages),
    true,
  );
});

test("verifyProof returns false for tampered revealed messages", () => {
  const fixture = loadFixture();
  const revealedMessages = fixture.revealedMessages.map((message) => ({ ...message }));
  revealedMessages[1] = {
    ...revealedMessages[1],
    data: Buffer.from("1990-01-02"),
  };

  assert.equal(
    verifyProof(fixture.publicKey, fixture.proof, revealedMessages),
    false,
  );
});

test("verifyProof throws for malformed byte inputs", () => {
  const fixture = loadFixture();

  assert.throws(() =>
    verifyProof(new Uint8Array([1, 2, 3]), fixture.proof, fixture.revealedMessages),
  );
});
