{
  description = "luminate workspace dev shell";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    systems.url = "github:nix-systems/default-linux";
  };

  outputs = inputs @ {
    self,
    nixpkgs,
    systems,
    ...
  }: let
    eachSystem = nixpkgs.lib.genAttrs (import systems);

    pkgsFor = system:
      import nixpkgs {
        inherit system;
        overlays = [];
      };
  in {
    devShells = eachSystem (system: let
      pkgs = pkgsFor system;
    in {
      default = pkgs.mkShell {
        name = "luminate devShell";
        nativeBuildInputs = with pkgs; [
          # Rust toolchain
          cargo
          rustc
          clippy
          rustfmt
          rust-analyzer

          # Build tooling
          pkg-config

          # winit (X11 / Wayland)
          libxkbcommon
          libx11
          libxcursor
          libxi
          libxrandr
          wayland
          wayland-protocols
          wayland-scanner

          # wgpu
          vulkan-loader
          mesa # Mesa drivers, otherwise the Vulkan backend cannot see the system driver

          # System
          dbus
          openssl

          # CI tooling
          cargo-deny
        ];

        LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath (with pkgs; [
          vulkan-loader
          mesa
          libxkbcommon
          libx11
          libxcursor
          libxi
          libxrandr
          wayland
          wayland-protocols
        ])}";
        env.RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
      };
    });

    formatter = eachSystem (system: (pkgsFor system).alejandra);
  };
}
