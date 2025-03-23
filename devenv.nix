{ pkgs, lib, config, inputs, ... }:

let
  unstable = import inputs.nixpkgs-unstable { system = pkgs.stdenv.system; };
in {
  languages = {
    rust = {
      enable = true;
      channel = "stable";
      components = [ "rustc" "cargo" "clippy" "rustfmt" "rust-analyzer" ];
    };
    python.enable = true;
    perl.enable = true;
  };

  services.postgres = {
    enable = true;
    package = pkgs.postgresql_16;
    initialDatabases = [{
      name = "malbox_db";
    }];
    initialScript = ''
      CREATE ROLE postgres WITH LOGIN SUPERUSER PASSWORD 'password';
    '';
    settings = {
      listen_addresses = lib.mkForce "127.0.0.1";
      max_connections = 100;
    };
  };

  env = {
    LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
    BINDGEN_EXTRA_CLANG_ARGS = ''-I"${pkgs.glibc.dev}/include"'';
    LD_LIBRARY_PATH = lib.makeLibraryPath [ pkgs.llvm_18 pkgs.clang_18 pkgs.libclang.lib ];
    DATABASE_URL = "postgres://postgres@localhost:5432/malbox_db";
  };

  packages = with pkgs; [
    nixd
    sqlx-cli
    openssl
    cargo-watch
    file
    libGL
    glib
    flex
    bison
    dtc
    zlib
    pixman
    python311Packages.sphinx
    python311Packages.sphinx-rtd-theme
    python311Packages.ninja
    libvirt
    llvm_18
    libllvm
    clang_18
    glibc
    packer
    terraform
    unstable.cocogitto
  ];

  processes = {
    rust-test-watch.exec = "cargo watch -x test";
    rust-check-watch.exec = "cargo watch -x check";
    rust-build-watch.exec = "cargo watch -x build";
  };

  scripts = {
    check.exec = "cargo check";
    test.exec = "cargo test";
    build.exec = "cargo build";
    clean.exec = "cargo clean";
    run.exec = "cargo run";
    docs.exec = ''
      echo "Building documentation..."
      cargo doc --no-deps
      python -m sphinx.cmd.build docs docs/_build
    '';
    fmt.exec = ''
      echo "Formatting code..."
      cargo fmt --all
    '';
    lint.exec = ''
      echo "Running clippy..."
      cargo clippy -- -D warnings
    '';
    db-setup.exec = ''
      echo "Setting up database..."
      sqlx database create
      sqlx migrate run
    '';
    db-reset.exec = ''
      echo "Resetting database..."
      sqlx database reset
    '';
  };

  pre-commit.hooks = {
    clippy.enable = true;
    rustfmt.enable = true;
    cargo-check.enable = true;
    shellcheck.enable = true;
  };

  enterTest = ''
    echo "Running environment tests..."
    cargo --version
    rustc --version
    python3 --version
    perl --version
    echo "Testing LLVM/Clang setup..."
    echo $LIBCLANG_PATH | grep "${pkgs.libclang.lib}/lib"
    echo $LD_LIBRARY_PATH | grep "${pkgs.llvm_18}/lib"
  '';

  enterShell = ''
    echo "🦀 Rust development environment initialized"
    echo "Available commands:"
    echo " - check        : Run cargo check"
    echo " - test         : Run cargo test"
    echo " - build        : Run cargo build"
    echo " - docs         : Build documentation"
    echo " - fmt          : Format code"
    echo " - lint         : Run clippy"
    echo " - db-setup     : Create and setup database"
    echo " - db-reset     : Reset database and run migrations"
    echo ""
    echo "Watching processes available:"
    echo " - rust-test-watch"
    echo " - rust-check-watch"
    echo " - rust-build-watch"
  '';
}
