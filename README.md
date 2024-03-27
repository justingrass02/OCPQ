## Installation
You can download the automatically cross-compiled installers for the latest release from [__github.com/aarkue/ocedeclare/releases/latest__](https://github.com/aarkue/ocedeclare/releases/latest).

The following installer formats are available:
- `[...].AppImage` for Linux (__Recommended for Linux__)
- `[...]-setup.exe` for Windows (__Recommended for Windows__)
- `[...].dmg` for MacOS (__Recommended for Mac__)
- `[...].deb` for Linux (Debian)
- `[...].msi` for Windows
- `[...].app.tar.gz` for MacOS

### Docker

Alternatively, you can also easily build and run the project locally using Docker.
This will start a local web server for the backend and the frontend.
Once the container is running, you can open [http://localhost:4567/](http://localhost:4567/) in your browser for the tool frontend.

#### Docker Compose
Run `docker compose up --build` in the project root.


#### Docker Files

- __backend__:
  1. First build using `sudo docker build ./backend -t ocedeclare-backend`
  2. Then run with `docker run --init -p 3000:3000 ocedeclare-backend`
- __frontend__:
  1. First build using `sudo docker build ./frontend -t ocedeclare-frontend`
  2. Then run with `sudo docker run --init -p 4567:4567 ocedeclare-backend`


## Usage
