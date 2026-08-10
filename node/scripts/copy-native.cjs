const fs = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const targetDir = process.env.CARGO_TARGET_DIR
  ? path.resolve(process.env.CARGO_TARGET_DIR)
  : path.join(root, "native", "target");

const extensionByPlatform = {
  darwin: "dylib",
  linux: "so",
  win32: "dll",
};

const platformDirByTarget = {
  "darwin-arm64": "darwin_arm64",
  "linux-x64": "linux_amd64",
};

const extension = extensionByPlatform[process.platform];
if (!extension) {
  throw new Error(`unsupported platform: ${process.platform}`);
}

const platformKey = `${process.platform}-${process.arch}`;
const platformDir = platformDirByTarget[platformKey];
if (!platformDir) {
  throw new Error(
    `unsupported Node native target: ${platformKey} (supported: darwin-arm64, linux-x64)`,
  );
}

const source = path.join(targetDir, "release", `libbbsplus_node.${extension}`);
const destinationDir = path.join(root, "native", platformDir);
const destination = path.join(destinationDir, "bbsplus_node.node");
const legacyDestination = path.join(root, "native", "bbsplus_node.node");

fs.mkdirSync(destinationDir, { recursive: true });
fs.copyFileSync(source, destination);

if (fs.existsSync(legacyDestination)) {
  fs.unlinkSync(legacyDestination);
}
