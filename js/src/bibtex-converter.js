import { BibLatexExporter, CSLParser } from "biblatex-csl-converter";

function cslToBibtex(cslJson) {
  const parser = new CSLParser(cslJson);
  const items = parser.parse();
  const exporter = new BibLatexExporter(items);
  return exporter.parse();
}

// Explicit CommonJS export
if (typeof module !== "undefined" && module.exports) {
  module.exports = { cslToBibtex };
} else if (typeof exports !== "undefined") {
  exports.cslToBibtex = cslToBibtex;
} else {
  // Fallback for other environments
  globalThis.cslToBibtex = cslToBibtex;
}
