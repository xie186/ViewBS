# ViewBS Docker Image

This Docker image builds the Rust `ViewBS` binary in a Rust builder stage and copies only the compiled executable into a small Debian runtime image.

Build from the repository root:

```bash
docker build -f ViewBSdocker/Dockerfile -t viewbs:rust .
```

Run ViewBS:

```bash
docker run --rm viewbs:rust --help
docker run --rm -v "$PWD":/data -w /data viewbs:rust GlobalMethLev --help
```

The runtime image is intended for the Rust rewrite and does not install external plotting or scripting runtimes. Plot generation is handled inside the `ViewBS` binary.
