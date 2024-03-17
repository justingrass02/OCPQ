## Docker

- __backend__:
  1. First build using `sudo docker build ./backend -t ocedeclare-backend`
  2. Then run with `docker run --init -p 3000:3000 ocedeclare-backend`
- __frontend__:
  1. First build using `sudo docker build ./frontend -t ocedeclare-frontend`
  2. Then run with `sudo docker run --init -p 4567:4567 ocedeclare-backend`
