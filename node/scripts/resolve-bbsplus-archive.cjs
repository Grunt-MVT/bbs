const path = require("node:path");

const libDir = process.env.BBSPLUS_LIB_DIR
  ? path.resolve(process.env.BBSPLUS_LIB_DIR)
  : path.resolve(__dirname, "../../../target/release");

process.stdout.write(path.join(libDir, "libbbsplus.a"));
