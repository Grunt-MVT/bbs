const fs = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");

const platformDirByTarget = {
  "darwin-arm64": "darwin_arm64",
  "linux-x64": "linux_amd64",
};

const platformKey = `${process.platform}-${process.arch}`;
const platformDir = platformDirByTarget[platformKey];
if (!platformDir) {
  throw new Error(
    `unsupported Node native target: ${platformKey} (supported: darwin-arm64, linux-x64)`,
  );
}

const sourceCandidates = [
  path.join(root, "native", "build", "Release", "bbsplus_node.node"),
  path.join(root, "native", "build", "Debug", "bbsplus_node.node"),
];
const source = sourceCandidates.find((candidate) => fs.existsSync(candidate));
if (!source) {
  throw new Error(
    `missing node-gyp output; looked for:\n${sourceCandidates.join("\n")}`,
  );
}

const destinationDir = path.join(root, "native", platformDir);
const destination = path.join(destinationDir, "bbsplus_node.node");
const legacyDestination = path.join(root, "native", "bbsplus_node.node");

fs.mkdirSync(destinationDir, { recursive: true });
fs.copyFileSync(source, destination);

if (fs.existsSync(legacyDestination)) {
  fs.unlinkSync(legacyDestination);
}
