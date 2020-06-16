//@ts-check
"use strict";

const { $, info, chalk } = require("@itchio/bob");
const { readFileSync, writeFileSync } = require("fs");

function main() {
  const { version } = require("../package.json");
  info(`Bumped to version ${chalk.yellow(version)}`);

  info("Editing Cargo.toml...");
  let contents = readFileSync("Cargo.toml", { encoding: "utf8" });
  let lines = contents.split("\n");
  let foundVersion = false;
  for (let i = 0; i < lines.length; i++) {
    let line = lines[i];
    if (/version = ".*"/.test(line)) {
      lines[i] = `version = ${JSON.stringify(version)}`;
      foundVersion = true;
      break;
    }
  }
  if (!foundVersion) {
    throw new Error("Could not find version line in Cargo.toml!");
  }

  contents = lines.join("\n");
  writeFileSync("Cargo.toml", contents, { encoding: "utf8" });

  info("Cargo checking...");
  $(`cargo check`);

  info("Adding files to git");
  $(`git add -A Cargo.toml Cargo.lock`);
}

main();
