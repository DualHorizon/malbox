{
  inputs = {
    nixpkgs.url = "github:cachix/devenv-nixpkgs/rolling";
    systems.url = "github:nix-systems/default";
    devenv.url = "github:cachix/devenv";
    devenv.inputs.nixpkgs.follows = "nixpkgs";
    fenix.url = "github:nix-community/fenix";
    fenix.inputs = { nixpkgs.follows = "nixpkgs"; };
  };

  nixConfig = {
    extra-trusted-public-keys = "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw=";
    extra-substituters = "https://devenv.cachix.org";
  };

  outputs =
    { self
    , nixpkgs
    , devenv
    , systems
    , ...
    } @ inputs:
    let
      forEachSystem = nixpkgs.lib.genAttrs (import systems);
    in
    {
      packages = forEachSystem (system: {
        devenv-up = self.devShells.${system}.default.config.procfileScript;
      });

      devShells =
        forEachSystem
          (system:
            let
              pkgs = nixpkgs.legacyPackages.${system};
            in
            {
              default = devenv.lib.mkShell {
                inherit inputs pkgs;
                modules = [
                  {
                    languages.rust = {
                      enable = true;
                      channel = "stable";
                      components = [
                        "rustc"
                        "cargo"
                        "clippy"
                        "rustfmt"
                        "rust-analyzer"
                      ];
                    };
                    languages.python = {
                      enable = true;
                    };
                    languages.perl = {
                      enable = true;
                    };
                    packages = with pkgs; [ sqlx-cli openssl cargo-watch file libGL glib flex bison dtc zlib pixman python311Packages.sphinx python311Packages.sphinx-rtd-theme python311Packages.ninja libvirt file llvm_18 libllvm clang_18];
                  enterShell = ''
                    export LIBCLANG_PATH="${pkgs.libclang.lib}/lib"
                    export BINDGEN_EXTRA_CLANG_ARGS="$(${pkgs.llvm_18}/bin/llvm-config --cxxflags)"
                    export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath [
                      pkgs.llvm_18
                      pkgs.clang_18
                      pkgs.libclang.lib
                    ]}''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
                  '';
                  }
                ];
              };
            });
    };
}
