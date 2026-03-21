#!/usr/bin/env bash

root="$(picoref root)"
cd "$root" || exit
RG_DEFAULT_COMMAND="rg -t toml -i -l"
FZF_DEFAULT_COMMAND="rg -t toml --files | xargs -I% basename % .toml" fzf \
  -m \
  -e \
  --ansi \
  --disabled \
  --reverse \
  --bind "ctrl-a:select-all" \
  --bind "f12:execute-silent:(subl -b {})" \
  --bind "change:reload:$RG_DEFAULT_COMMAND {q} | xargs -I% basename % .toml" \
  --preview 'picoref markdown {} | glow --style auto'
