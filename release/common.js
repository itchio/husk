//@ts-check
"use strict";

const childProcess = require("child_process");

let verbose = false;

/**
 * @returns {boolean}
 */
function isVerbose() {
  return verbose;
}

/**
 * @param {boolean} v
 */
function setVerbose(v) {
  verbose = v;
}

const chalk = {
  colors: {
    green: "\x1b[1;32;40m",
    yellow: "\x1b[1;33;40m",
    blue: "\x1b[1;34;40m",
    magenta: "\x1b[1;35;40m",
    cyan: "\x1b[1;36;40m",
    reset: "\x1b[0;0;0m",
  },
  /**
   * @param {any} s
   */
  green: function (s) {
    return `${chalk.colors.green}${s}${chalk.colors.reset}`;
  },
  /**
   * @param {any} s
   */
  yellow: function (s) {
    return `${chalk.colors.yellow}${s}${chalk.colors.reset}`;
  },
  /**
   * @param {any} s
   */
  blue: function (s) {
    return `${chalk.colors.blue}${s}${chalk.colors.reset}`;
  },
  /**
   * @param {any} s
   */
  magenta: function (s) {
    return `${chalk.colors.magenta}${s}${chalk.colors.reset}`;
  },
  /**
   * @param {any} s
   */
  cyan: function (s) {
    return `${chalk.colors.cyan}${s}${chalk.colors.reset}`;
  },
};

/**
 * @param {string} line
 */
function info(line) {
  console.log(chalk.blue(`⚾ ${line}`));
}

/**
 * @param {string} line
 */
function header(line) {
  let bar = "―".repeat(line.length + 2);

  console.log();
  console.log(chalk.blue(bar));
  console.log(chalk.blue(` ${line} `));
  console.log(chalk.blue(bar));
  console.log();
}

/**
 * @param {string} cmd A command to launch (through bash)
 */
function system(cmd) {
  if (process.platform === "win32") {
    childProcess.execSync("bash", {
      stdio: ["pipe", "inherit", "inherit"],
      input: cmd,
    });
  } else {
    childProcess.execSync(cmd, {
      stdio: "inherit",
    });
  }
}

/**
 * @param {string} cmd Command to run
 */
function $(cmd) {
  console.log(chalk.yellow(`✨ ${cmd}`));
  system(cmd);
}

/**
 * @param {string} cmd
 * @param {{silent?: boolean}} [opts]
 * @returns {string} stdout
 */
function $$(cmd, opts) {
  if (!opts) {
    opts = {};
  }
  if (!opts.silent) {
    console.log(chalk.yellow(`✨ ${cmd}`));
  }
  const cp = require("child_process");
  return cp.execSync(cmd, {
    stdio: ["inherit", "pipe", "inherit"],
    encoding: "utf8",
  });
}

function debug() {
  if (!verbose) {
    return;
  }
  // @ts-ignore
  console.log.apply(console, arguments);
}

/**
 * @returns {string}
 */
function detectOS() {
  switch (process.platform) {
    case "win32":
      return "windows";
    case "darwin":
      return "darwin";
    case "linux":
      return "linux";
    default:
      throw new Error(`Unsupported process.platform: ${process.platform}`);
  }
}

/**
 * Exports an environment variable
 * @param {string} k
 * @param {string} v
 */
function setenv(k, v) {
  console.log(`export ${chalk.green(k)}=${chalk.yellow(v)}`);
  process.env[k] = v;
}

/**
 * @param {number} b An amount of bytes
 * @returns {string} A human-readable size
 */
function formatSize(b) {
  let KiB = 1024;
  let MiB = 1024 * KiB;

  if (b > MiB) {
    return `${(b / MiB).toFixed(2)} MiB`;
  } else if (b > KiB) {
    return `${(b / KiB).toFixed(0)} KiB`;
  } else {
    return `${b} B`;
  }
}

/**
 * @param {number} x A number in the [0, 1] range
 * @returns {string} That number formatted as a percentage
 */
function formatPercent(x) {
  return `${(x * 100).toFixed(2)}%`;
}

/**
 * Returns the size of a file in bytes
 * @param {string} path The path of the file
 * @returns {number} The size of `path` in bytes
 */
function sizeof(path) {
  const { statSync } = require("fs");
  const stats = statSync(path);
  return stats.size;
}

module.exports = {
  info,
  header,
  chalk,
  system,
  debug,
  $,
  $$,
  setVerbose,
  isVerbose,
  detectOS,
  setenv,
  sizeof,
  formatSize,
  formatPercent,
};
