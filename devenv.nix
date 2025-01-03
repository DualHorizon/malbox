{ pkgs, lib, config, inputs, ... }: {
  # Enable languages
  languages = {
    rust = {
      enable = true;
      channel = "stable";
      components = [ "rustc" "cargo" "clippy" "rustfmt" "rust-analyzer" ];
    };
    python.enable = true;
    perl.enable = true;
  };

  # Environment variables
  env = {
    LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
    BINDGEN_EXTRA_CLANG_ARGS = ''-I"${pkgs.glibc.dev}/include"'';
    LD_LIBRARY_PATH = lib.makeLibraryPath [ pkgs.llvm_18 pkgs.clang_18 pkgs.libclang.lib ];
    DATABASE_URL = "postgres://postgres:password@localhost:5432/malbox_db";
  };

  # Required packages
  packages = with pkgs; [
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
    file
    llvm_18
    libllvm
    clang_18
    glibc
  ];

  # Docker configuration
  containers.enable = true;

  # Development processes
  processes = {
    # Watch for Rust file changes and run tests
    rust-test-watch.exec = "cargo watch -x test";
    # Watch for changes and run cargo check
    rust-check-watch.exec = "cargo watch -x check";
    # Run cargo build in watch mode
    rust-build-watch.exec = "cargo watch -x build";
    # Start PostgreSQL container
    postgres.exec = ''
      if ! docker ps -a | grep -q malbox-postgres-16; then
        docker run --name malbox-postgres-16 \
          -e POSTGRES_PASSWORD=password \
          -e POSTGRES_DB=malbox_db \
          -d -p 5432:5432 postgres:16
      elif ! docker ps | grep -q malbox-postgres-16; then
        docker start malbox-postgres-16
      fi
    '';
  };

  # Scripts for common tasks
  scripts = {
    check.exec = "cargo check";
    test.exec = "cargo test";
    build.exec = "cargo build";
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
    # Database management scripts
    db-setup.exec = ''
      echo "Setting up database..."
      docker exec -it malbox-postgres-16 createdb -U postgres malbox_db || true
      sqlx database create
      sqlx migrate run
    '';
    db-reset.exec = ''
      echo "Resetting database..."
      sqlx database drop
      sqlx database create
      sqlx migrate run
    '';
  };

  # Pre-commit hooks
  pre-commit.hooks = {
    clippy.enable = true;
    rustfmt.enable = true;
    cargo-check.enable = true;
    shellcheck.enable = true;
  };

  # Tests to run when entering the environment
  enterTest = ''
    echo "Running environment tests..."
    cargo --version
    rustc --version
    python3 --version
    perl --version
    docker --version
    echo "Testing LLVM/Clang setup..."
    echo $LIBCLANG_PATH | grep "${pkgs.libclang.lib}/lib"
    echo $LD_LIBRARY_PATH | grep "${pkgs.llvm_18}/lib"
  '';

  # Shell initialization
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
    echo " - postgres     : Starts PostgreSQL container"

    # Start PostgreSQL container if not running
    if ! docker ps | grep -q malbox-postgres-16; then
      echo "Starting PostgreSQL container..."
      devenv processes start postgres
    fi
  '';
}
