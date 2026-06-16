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

const extension = extensionByPlatform[process.platform];
if (!extension) {
  throw new Error(`unsupported platform: ${process.platform}`);
}

const source = path.join(targetDir, "release", `libbbsplus_node.${extension}`);
const destination = path.join(root, "native", "bbsplus_node.node");

fs.copyFileSync(source, destination);
