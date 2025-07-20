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

      rustToolchain = pkgs.fenix.complete.withComponents [
        "cargo"
        "clippy"
        "rustc"
        "rustfmt"
        "rust-src"
        "rustc-codegen-cranelift-preview"
      ];

      rustPlatform = pkgs.makeRustPlatform {inherit (rustToolchain) cargo rustc;};

      craneLib =
        (crane.mkLib pkgs).overrideToolchain
        rustToolchain;

      src = pkgs.lib.fileset.toSource {
        root = ./.;
        fileset = pkgs.lib.fileset.unions [
          ./assets
          (craneLib.fileset.commonCargoSources ./.)
        ];
      };

      args = {
        inherit src;
        strictDeps = true;
        buildInputs = with pkgs; [libxkbcommon pipewire];
        nativeBuildInputs = with pkgs; [
          rustPlatform.bindgenHook
          pkg-config
          mold
          makeWrapper
        ];
      };

      cargoArtifacts = craneLib.buildDepsOnly args;
      cargoClippyExtraArgs = "--all-targets -- --deny warnings";

      package = craneLib.buildPackage (args
        // {
          inherit cargoArtifacts;
          postInstall = ''wrapProgram "$out/bin/status" --prefix LD_LIBRARY_PATH : "${libraryPath}"'';
        });

      libraryPath = pkgs.lib.makeLibraryPath (with pkgs; [
        pipewire
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
