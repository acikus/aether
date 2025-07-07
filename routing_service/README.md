# Routing Service

This folder contains a small Flask application that proxies requests to a
running Valhalla server.  The service exposes two endpoints:

- `/route`  – compute a single route between locations.
- `/matrix` – compute a distance matrix between locations.

The application relies on the Valhalla server being available locally (default
`http://localhost:8080`).  The base URL can be configured with the
`VALHALLA_URL` environment variable.

## Running

Install Python dependencies and start the Flask server:

```bash
pip install -r requirements.txt
python -m routing_service.app
```

See `test_valhalla.py` for a minimal example of calling Valhalla directly.
