## Docker


Note: Currently the API web server infrastructure used in the docker version only supports hardcoded included OCEL2 files.
Make sure to include at least some of the following OCEL2 files in  `./backend/data/`: `order-management.json`, `ocel2-p2p.json`, `ContainerLogistics.json`(available at https://www.ocel-standard.org/event-logs/overview/).

### Docker Compose
Run `docker compose up --build` in the project root.


### Docker Files

- __backend__:
  1. First build using `sudo docker build ./backend -t ocedeclare-backend`
  2. Then run with `docker run --init -p 3000:3000 ocedeclare-backend`
- __frontend__:
  1. First build using `sudo docker build ./frontend -t ocedeclare-frontend`
  2. Then run with `sudo docker run --init -p 4567:4567 ocedeclare-backend`

