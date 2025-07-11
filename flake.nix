{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    nixpkgs,
    crane,
    fenix,
    ...
  }: let
    systems = ["x86_64-linux"];
    perSystem = f:
      nixpkgs.lib.foldAttrs nixpkgs.lib.mergeAttrs {}
      (map (s: nixpkgs.lib.mapAttrs (_: v: {${s} = v;}) (f s)) systems);
  in
    perSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [fenix.overlays.default];
      };

      craneLib =
        (crane.mkLib pkgs).overrideToolchain
        (pkgs.fenix.complete.withComponents [
          "cargo"
          "clippy"
          "rustc"
          "rust-src"
          "rustc-codegen-cranelift-preview"
        ]);

      src = craneLib.cleanCargoSource ./.;

      args = {
        inherit src;
        strictDeps = true;
        nativeBuildInputs = with pkgs; [pkg-config mold];
        buildInputs = with pkgs; [libxkbcommon];
      };

      cargoArtifacts = craneLib.buildDepsOnly args;
      cargoClippyExtraArgs = "--all-targets -- --deny warnings";
      package = craneLib.buildPackage (args // {inherit cargoArtifacts;});

      libraryPath = pkgs.lib.makeLibraryPath (with pkgs; [
        libxkbcommon
        vulkan-loader
        wayland
      ]);
    in {
      devShells.default = craneLib.devShell {
        packages = with pkgs; [watchexec] ++ (with args; (nativeBuildInputs ++ buildInputs));
        LD_LIBRARY_PATH = libraryPath;
      };

      checks = {
        inherit package;

        fmt = craneLib.cargoFmt (args // {inherit cargoArtifacts;});
        fmt-toml = craneLib.taploFmt {src = pkgs.lib.sources.sourceFilesBySuffices src [".toml"];};
        test = craneLib.cargoTest (args // {inherit cargoArtifacts;});
        clippy = craneLib.cargoClippy (args // {inherit cargoArtifacts cargoClippyExtraArgs;});
      };

      packages.default = package;
    });
}
