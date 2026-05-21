#!/usr/bin/env bash

local_only_repo_root() {
  git rev-parse --show-toplevel
}

local_only_die() {
  echo "local-only: $*" >&2
  exit 1
}

local_only_require_cmd() {
  local cmd="$1"
  command -v "$cmd" >/dev/null 2>&1 || local_only_die "required command not found: $cmd"
}

local_only_require_clean_worktree() {
  local status
  status="$(git status --porcelain)"
  if [[ -n "$status" ]]; then
    echo "$status" >&2
    local_only_die "working tree must be clean"
  fi
}

local_only_default_base_ref() {
  if git rev-parse --verify upstream/master >/dev/null 2>&1; then
    echo "upstream/master"
  elif git rev-parse --verify origin/master >/dev/null 2>&1; then
    echo "origin/master"
  elif git rev-parse --verify master >/dev/null 2>&1; then
    echo "master"
  else
    local_only_die "could not find upstream/master, origin/master, or master"
  fi
}

local_only_patch_dir() {
  local root
  root="$(local_only_repo_root)"
  echo "$root/patches/local-only"
}

local_only_series_file() {
  local patch_dir="$1"
  echo "$patch_dir/series"
}

local_only_print_header() {
  local title="$1"
  echo
  echo "==> $title"
}
