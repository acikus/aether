import requests


def test_valhalla_route():
    """Example request to a local Valhalla server."""
    url = "http://localhost:8080/route"
    payload = {
        "locations": [
            {"lat": 44.787197, "lon": 20.457273},  # Beograd
            {"lat": 44.804010, "lon": 20.465130},  # Novi Beograd
        ],
        "costing": "auto",
        "directions_options": {"units": "kilometers"},
    }
    headers = {"Content-Type": "application/json"}
    response = requests.post(url, json=payload, headers=headers)
    if response.status_code == 200:
        data = response.json()
        summary = data.get("trip", {}).get("summary", {})
        print(f"Distance: {summary.get('length')} km")
        print(f"Time: {summary.get('time')} sec")
        return data
    else:
        print(f"Error {response.status_code}: {response.text}")
        return None


if __name__ == "__main__":
    test_valhalla_route()
