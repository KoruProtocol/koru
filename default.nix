
let

  holonixRev = "9c9a5a00dc05b0825841fae4ff8181182d9949ce";

  holonixPath = builtins.fetchTarball "https://github.com/holochain/holonix/archive/${holonixRev}.tar.gz";
  holonix = import (holonixPath) {
    holochainVersionId = "v0_0_121";
  };
  nixpkgs = holonix.pkgs;
in nixpkgs.mkShell {
  inputsFrom = [ holonix.main ];
  buildInputs = with nixpkgs; [
    binaryen
    nodejs-16_x
  ];
}  

#to update holochain see https://github.com/holochain-open-dev/wiki/wiki/How-to-upgrade-the-Holochain-version-of-a-project-with-Nix