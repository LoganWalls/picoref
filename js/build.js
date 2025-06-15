#!/usr/bin/env node

require("esbuild")
  .build({
    entryPoints: ["src/bibtex-converter.js"],
    bundle: true,
    platform: "node",
    format: "iife",
    target: "es2020",
    outfile: "dist/bibtex-converter.bundle.js",
    globalName: "BibtexConverter",
  })
  .catch((e) => {
    console.log(e);
    process.exit(1);
  });
console.log("esbuild completed");
