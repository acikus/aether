import os
from typing import List, Tuple, Dict

import requests


class ValhallaClient:
    """Simple client for interacting with a local Valhalla routing service."""

    def __init__(self, base_url: str | None = None) -> None:
        self.base_url = base_url or os.getenv("VALHALLA_URL", "http://localhost:8080")

    def route(
        self,
        locations: List[Tuple[float, float]],
        costing: str = "auto",
        units: str = "kilometers",
    ) -> Dict:
        """Request a route between locations.

        Parameters
        ----------
        locations:
            List of (lat, lon) tuples defining the route stops.
        costing:
            Valhalla costing model, e.g. "auto" or "bicycle".
        units:
            Distance units returned by Valhalla.
        """
        payload = {
            "locations": [{"lat": lat, "lon": lon} for lat, lon in locations],
            "costing": costing,
            "directions_options": {"units": units},
        }
        resp = requests.post(f"{self.base_url}/route", json=payload)
        resp.raise_for_status()
        return resp.json()

    def matrix(
        self,
        sources: List[Tuple[float, float]],
        targets: List[Tuple[float, float]] | None = None,
        costing: str = "auto",
        units: str = "kilometers",
    ) -> Dict:
        """Request a distance matrix between locations."""
        if targets is None:
            targets = sources
        payload = {
            "sources": [{"lat": lat, "lon": lon} for lat, lon in sources],
            "targets": [{"lat": lat, "lon": lon} for lat, lon in targets],
            "costing": costing,
            "directions_options": {"units": units},
        }
        resp = requests.post(f"{self.base_url}/matrix", json=payload)
        resp.raise_for_status()
        return resp.json()
