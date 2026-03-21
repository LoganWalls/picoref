#! /usr/bin/env nix-shell
#! nix-shell -i bash -p pnpm nodejs
set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)
picoref="$script_dir/target/release/picoref"
json_path="$($picoref root)/references.json"
outpath="$1"
"$picoref" export "$json_path" \
  && pnpx @citation-js/cli \
    -i "$json_path" -s bibtex -f string -o "${outpath%.*}" \
  && sed -i 's/&/\&/g' "$outpath" \
  && sed -i '/^[[:space:]]*url[[:space:]]*=/d; /^[[:space:]]*doi[[:space:]]*=/d' "$outpath" \
  && sed -i '0,/Walls2024Regret/!{0,/Walls2024Regret/s/Walls2024Regret/Walls2024Regret-2/}' "$outpath"
