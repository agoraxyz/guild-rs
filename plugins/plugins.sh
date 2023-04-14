#!/bin/sh

dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

for d in $dir/*/
do
  case "$1" in
    "fmt")
      echo "Formatting Rust crate in directory $d ..."
      (cd "$d" && cargo fmt)
      ;;
    "build")
      echo "Building Rust crate in directory $d ..."
      (cd "$d" && cargo b -r)
      ;;
    "clean")
      echo "Cleaning Rust crate in directory $d ..."
      (cd "$d" && cargo clean)
      ;;
    "check")
      echo "Running Check for Rust crate in directory $d ..."
      (cd "$d" && cargo check)
      ;;
    "clippy")
      echo "Running Clippy for Rust crate in directory $d ..."
      (cd "$d" && cargo clippy)
      ;;
    "test")
      echo "Running tests for Rust crate in directory $d ..."
      (cd "$d" && cargo t --all-features)
      ;;
  esac
done

exit 0
