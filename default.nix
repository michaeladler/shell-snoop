{ stdenv, lib, cmake, pkg-config, libcap_ng }:

stdenv.mkDerivation rec {
  pname = "shell-snoop";
  version = lib.fileContents  ./VERSION;

  src = lib.cleanSource ./.;

  nativeBuildInputs = [ cmake pkg-config ];
  buildInputs = [ libcap_ng ];

  meta = with lib; {
    homepage = "https://github.com/michaeladler/shell-snoop";
    description = "figure out the exact command which was used to run a child process in a shell";
    platforms = platforms.all;
    license = licenses.asl20;
  };
}
