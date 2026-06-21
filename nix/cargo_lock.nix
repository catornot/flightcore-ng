{ ... }:
{
  lockFile = ../Cargo.lock;
  # outputHashes = {
  #   "nix-compat-0.1.0" = "sha256-vN5G5+2N47M2sa9fu6fVRsC0fHe8qpzxgh2u38mKtC0=";
  #   "nix-compat-derive-0.1.0" = lib.fakeHash;
  #   "snix-eval-0.1.0" = lib.fakeHash;
  #   "snix-eval-builtin-macros-0.0.1" = lib.fakeHash;
  # };
  # TODO: this is not good, it doesn't use a hash
  allowBuiltinFetchGit = true;
}
