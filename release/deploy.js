//@ts-check
"use strict";

const { readFileSync } = require("fs");
const { $, chalk, info, header } = require("@itchio/bob");

function main() {
  let expectedManifestName = "@itchio/husk";

  try {
    let manifest = JSON.parse(
      readFileSync("./package.json", { encoding: "utf8" })
    );
    if (manifest.name !== expectedManifestName) {
      info(
        `Expected manifest name ${chalk.yellow(
          expectedManifestName
        )}, got ${chalk.yellow(manifest.name)}`
      );
      throw new Error("Unexpected manifest name");
    }
  } catch (e) {
    throw new Error(
      `Script must be invoked as 'node release/deploy.js', from the root repository folder.\n` +
        `Was invoked from ${chalk.yellow(process.cwd())}.`
    );
  }

  header("Gathering information");
  $(`node --version`);

  let tag = process.env.CI_COMMIT_TAG;
  if (!tag) {
    throw new Error(`$CI_COMMIT_TAG not set, bailing out`);
  }

  let matches = /v([0-9]+)\.([0-9]+)\.([0-9]+)/.exec(tag);
  if (!matches) {
    throw new Error(
      `Could not parse version ${chalk.yellow(
        tag
      )} - is it missing the 'v' prefix?`
    );
  }
  let [, major, minor, patch] = matches;
  info(`Releasing version ${chalk.yellow(`${major}.${minor}.${patch}`)}`);

  throw new Error(`${chalk.magenta("Deploy is a stub")} right now.`);
}

main();
