//@ts-check
"use strict";

const {
  mkdirSync,
  readFileSync,
  readdirSync,
  statSync,
} = require("fs");

const fs = require("fs");
// fix for nodejs 16, rmdirsync is both deprecated and no longer works on files
const rmdirSync = fs.rmSync || fs.rmdirSync;

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

  rmdirSync("./artifacts/tmp.zip", { recursive: true });
  const targets = readdirSync("./artifacts");
  info(`Will upload targets: ${targets.map(chalk.yellow).join(", ")}`);

  if (process.env.DRY_RUN) {
    info("Dry run, bailing out now");
    return;
  }

  header("Uploading native addons...");
  mkdirSync("./release-tools", { recursive: true });

  let toolRepo = `https://github.com/github-release/github-release`;
  let toolTag = `v0.8.1`;
  let toolUrl = `${toolRepo}/releases/download/${toolTag}/linux-amd64-github-release.bz2`;
  let ghr = `./release-tools/github-release`;
  try {
    statSync(ghr);
    info(`Using existing ${chalk.yellow(ghr)}...`);
  } catch (e) {
    info(`Downloading ${chalk.yellow(ghr)}`);
    $(`curl --location "${toolUrl}" | bunzip2 > ${ghr}`);
  }
  $(`chmod +x ${ghr}`);
  $(`${ghr} --version`);

  process.env.GITHUB_USER = "itchio";
  process.env.GITHUB_REPO = "husk";
  if (!process.env.GITHUB_TOKEN) {
    throw new Error(
      `${chalk.yellow(
        "$GITHUB_TOKEN"
      )} is unset, are you running this script outside of CI?`
    );
  }

  try {
    $(`${ghr} delete --tag "${tag}"`);
    info(`Probably replacing release`);
  } catch (e) {
    info(`Probably not replacing release`);
  }
  $(`${ghr} release --tag "${tag}"`);

  for (const target of targets) {
    let label = `Prebuilt library for ${target}`;
    $(
      `(cd ./artifacts; zip --display-dots --recurse-paths ./tmp.zip ./${target})`
    );
    // note: github-release says it can upload from stdin, but it can't
    $(
      [
        `${ghr} upload --tag "${tag}" --file "./artifacts/tmp.zip" --replace`,
        `--label "${label}" --name "${target}.zip"`,
      ].join(" ")
    );
    rmdirSync("./artifacts/tmp.zip", { recursive: true });
  }
}

main();
