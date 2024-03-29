//@ts-check
"use strict";

const {
  chalk,
  detectOS,
  downloadToStream,
  debug,
  setVerbose,
} = require("@itchio/bob");
const { resolve } = require("path");
const {
  readFileSync,
  createWriteStream,
  existsSync,
  renameSync,
  unlinkSync,
} = require("fs");

const fs = require("fs");
// fix for nodejs 16, rmdirsync is both deprecated and no longer works on files
const rmdirSync = fs.rmSync || fs.rmdirSync;

/**
 * @param {string[]} args
 */
async function main(args) {
  let start = Date.now();

  let opts = {
    os: detectOS(),
    arch: process.arch === "ia32" ? "i686" : "x86_64",
    force: false,
    userSpecifiedOS: false,
    userSpecifiedArch: false,
  };

  for (let i = 0; i < args.length; i++) {
    let arg = args[i];

    if (arg === "-v" || arg === "--verbose") {
      setVerbose(true);
    } else if (arg === "--arch") {
      i++;
      let v = args[i];
      opts.arch = v;
      opts.userSpecifiedArch = true;
    } else if (arg === "--os") {
      i++;
      let v = args[i];
      if (v === "windows" || v === "darwin" || v === "linux") {
        opts.os = v;
      } else {
        throw new Error(`Unsupported os ${chalk.yellow(v)}`);
      }
      opts.userSpecifiedOS = true;
    } else if (arg === "--force") {
      opts.force = true;
    } else {
      throw new Error(`Unknown argument ${arg}`);
    }
  }

  if (existsSync(".git") && !opts.force) {
    console.log(
      `In development (${chalk.yellow(
        `.git`
      )} found), skipping postinstall (Use ${chalk.yellow("--force")} to force)`
    );
    return;
  }

  let { version } = require("../package.json");

  if (opts.userSpecifiedOS) {
    console.log(`Using user-specified os ${chalk.yellow(opts.os)}`);
  } else {
    debug(`Using detected os ${chalk.yellow(opts.os)}`);
  }

  if (opts.userSpecifiedArch) {
    console.log(`Using user-specified arch ${chalk.yellow(opts.arch)}`);
  } else {
    debug(`Using detected arch ${chalk.yellow(opts.arch)}`);
  }

  let platform = `${opts.os}-${opts.arch}`;
  console.log(`husk ${chalk.yellow(version)} on ${chalk.yellow(platform)}`);

  let artifactsPath = "./artifacts";

  if (!(opts.userSpecifiedArch || opts.userSpecifiedOS)) {
    debug(
      `Platform is fully autodetected, probing ${chalk.yellow(artifactsPath)}`
    );
    if (shouldSkipDownload({ artifactsPath, version })) {
      debug(`Nothing to do`);
      return;
    }
  }

  let tag = `v${version}`;
  let url = `https://github.com/itchio/husk/releases/download/${tag}/${platform}.zip`;

  let output = `./tmp.zip`;
  let out = createWriteStream(output, { autoClose: true });

  console.log(`Downloading from ${chalk.yellow(url)}`);
  await downloadToStream(url, out);

  const extract = require("extract-zip");
  await extract(output, { dir: resolve(".") });

  unlinkSync(output);
  if (existsSync(artifactsPath)) {
    rmdirSync(artifactsPath, { recursive: true });
  }
  renameSync(platform, artifactsPath);

  let end = Date.now();
  let totalTime = `${((end - start) / 1000).toFixed(1)}s`;
  debug(`Total time: ${chalk.yellow(totalTime)}`);
}

/**
 * @param {{ artifactsPath: string; version: string; }} opts
 * @returns {boolean}
 */
function shouldSkipDownload(opts) {
  const { artifactsPath, version } = opts;

  let versionFilePath = `${artifactsPath}/version`;
  if (!existsSync(versionFilePath)) {
    debug(`Prebuilt libd doesn't exist on disk yet`);
    return false;
  }

  let installedVersion = readFileSync(versionFilePath, {
    encoding: "utf-8",
  }).trim();
  debug(`Prebuilt lib on disk has version ${chalk.yellow(installedVersion)}`);

  if (installedVersion !== version) {
    debug(`Prebuilt lib on disk is the wrong version`);
    return false;
  }

  return true;
}

main(process.argv.slice(2));
