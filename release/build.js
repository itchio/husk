//@ts-check
"use strict";

const {
  formatPercent,
  formatSize,
  sizeof,
  detectOS,
  setVerbose,
  header,
  debug,
  info,
  chalk,
  setenv,
  $,
  $$,
} = require("./common");
const { readFileSync, writeFileSync, mkdirSync, rmdirSync } = require("fs");

const DEFAULT_ARCH = "x86_64";
const CBINDGEN_VERSION = "0.14.2";

/**
 * @typedef OsInfo
 * @type {{
 *   architectures: {
 *     [key: string]: {
 *       triplet: string,
 *       prependPath?: string,
 *     }
 *   }
 * }}
 */

/**
 * @type {{[name: string]: OsInfo}}
 */
const OS_INFOS = {
  windows: {
    architectures: {
      i686: {
        triplet: "i686-pc-windows-gnu",
        prependPath: "/mingw32/bin",
      },
      x86_64: {
        triplet: "x86_64-pc-windows-gnu",
        prependPath: "/mingw64/bin",
      },
    },
  },
  linux: {
    architectures: {
      x86_64: {
        triplet: "x86_64-unknown-linux-gnu",
      },
    },
  },
  darwin: {
    architectures: {
      x86_64: {
        triplet: "x86_64-apple-darwin",
      },
    },
  },
};

/**
 * @param {string[]} args
 */
function main(args) {
  info(`Preparing to build ${chalk.blue("husk")}`);

  /**
   * @type {{
   *   os?: string,
   *   arch?: string,
   * }}
   */
  let opts = {};

  for (let i = 0; i < args.length; i++) {
    let arg = args[i];

    let matches = /^--(.*)$/.exec(arg);
    if (matches) {
      let k = matches[1];
      if (k == "verbose") {
        setVerbose(true);
        continue;
      }

      if (k === "os" || k === "arch") {
        i++;
        let v = args[i];
        opts[k] = v;
      }
    } else if (arg == "-v") {
      setVerbose(true);
    } else {
      throw new Error(`Unknown parameter: ${chalk.yellow(arg)}`);
    }
  }

  if (!opts.os) {
    opts.os = detectOS();
    console.log(
      `Using detected OS ${chalk.yellow(opts.os)} (use ${chalk.yellow(
        "--os"
      )} to override)`
    );
  } else {
    console.log(`Using specified OS ${chalk.yellow(opts.os)}`);
  }

  let osInfo = OS_INFOS[opts.os];
  debug({ osInfo });

  if (!opts.arch) {
    opts.arch = DEFAULT_ARCH;
    console.log(
      `Using default arch ${chalk.yellow(opts.arch)} (use ${chalk.yellow(
        "--arch"
      )} to override)`
    );
  } else {
    console.log(`Using specified arch ${chalk.yellow(opts.arch)}`);
  }

  let archInfo = osInfo.architectures[opts.arch];
  debug({ archInfo });
  if (!archInfo) {
    throw new Error(`Unsupported arch '${opts.arch}' for os '${opts.os}'`);
  }

  if (archInfo.prependPath) {
    if (opts.os === "windows") {
      let prependPath = $$(`cygpath -w ${archInfo.prependPath}`).trim();
      console.log(
        `Prepending ${chalk.yellow(archInfo.prependPath)} (aka ${chalk.yellow(
          prependPath
        )}) to $PATH`
      );
      process.env.PATH = `${prependPath};${process.env.PATH}`;
    } else {
      console.log(`Prepending ${chalk.yellow(archInfo.prependPath)} to $PATH`);
      process.env.PATH = `${archInfo.prependPath}:${process.env.PATH}`;
    }
  }

  header("Installing tooling");
  $(`node --version`);
  $(`go version`);
  $(`rustup -V`);

  let channel = "stable";
  let toolchain = `${channel}-${archInfo.triplet}`;

  $(`rustup toolchain install ${toolchain}`);

  info(`Installing cbindgen...`);
  $(`cargo install --version ${CBINDGEN_VERSION} cbindgen`);
  $(`cbindgen --version`);

  info(`Installing c-for-go...`);
  $(`go build github.com/xlab/c-for-go`);

  header(`Generating build artifacts`);
  info(`Building static library`);
  $(`cargo +${toolchain} build --release`);

  info(`Generating C headers from Rust`);
  $(`cbindgen --quiet --output ./include/husk.h`);

  info(`Generating cgo bindings from C headers`);
  $(`./c-for-go husk.yml`);

  info(`Generating artifacts`);
  rmdirSync("artifacts", { recursive: true });
  mkdirSync("artifacts", { recursive: true });
  $(`cp -rf lowhusk/ husk/ include/ artifacts/`);
  let libName = opts.os === "windows" ? "husk.lib" : "libhusk.a";
  mkdirSync("artifacts/lib", { recursive: true });

  let libPath = "artifacts/lib/libhusk.a";
  $(`cp -rf target/release/${libName} ${libPath}`);
  {
    info(`Stripping debug symbols (disable with --no-strip)`);

    let before = sizeof(libPath);

    // note: Linux & Windows (mingw64) support '--strip-debug' but macOS has big
    // BSD energy and only supports '-S'
    $(`strip -S ${libPath}`);

    let after = sizeof(libPath);

    console.log(
      `Before: ${chalk.yellow(formatSize(before))} ` +
        `After: ${chalk.yellow(formatSize(after))} ` +
        `(${chalk.yellow(formatPercent((before - after) / before))} reduction)`
    );
  }

  info(`Generating cflags/ldflags`);
  let prefix = process.cwd().replace(/\\/g, "/") + "/artifacts";
  {
    /** @type {import("fs").WriteFileOptions} */
    let writeOpts = { encoding: "utf-8" };

    {
      let cflags = `-I@prefix@/include`;
      writeFileSync(`${prefix}/cflags.txt`, cflags, writeOpts);
    }

    {
      let ldflags = `-L@prefix@/lib -lhusk`;
      if (opts.os === "windows") {
        let libs = "ws2_32 advapi32 ole32 shell32 userenv".split(" ");
        let libArgs = libs.map((x) => `-l${x}`);
        ldflags += ` ${libArgs.join(" ")}`;
      }
      writeFileSync(`${prefix}/ldflags.txt`, ldflags, writeOpts);
    }
  }

  info(`Building & running sample binary`);
  let cflags = readFileSync("./artifacts/cflags.txt", { encoding: "utf8" });
  setenv("CGO_CFLAGS", cflags.replace("@prefix@", prefix));
  let ldflags = readFileSync("./artifacts/ldflags.txt", { encoding: "utf8" });
  setenv("CGO_LDFLAGS", ldflags.replace("@prefix@", prefix));

  $(`go run main.go`);
}

main(process.argv.slice(2));
