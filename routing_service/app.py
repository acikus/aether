from flask import Flask, jsonify, request

from .valhalla_client import ValhallaClient

app = Flask(__name__)
client = ValhallaClient()


@app.route('/route', methods=['POST'])
def route():
    data = request.get_json(force=True)
    locations = [
        (loc['lat'], loc['lon']) for loc in data.get('locations', [])
    ]
    costing = data.get('costing', 'auto')
    units = data.get('units', 'kilometers')
    try:
        result = client.route(locations, costing=costing, units=units)
        return jsonify(result)
    except Exception as exc:
        return jsonify({'error': str(exc)}), 500


@app.route('/matrix', methods=['POST'])
def matrix():
    data = request.get_json(force=True)
    sources = [
        (loc['lat'], loc['lon']) for loc in data.get('sources', [])
    ]
    targets = data.get('targets')
    if targets is not None:
        targets = [(loc['lat'], loc['lon']) for loc in targets]
    costing = data.get('costing', 'auto')
    units = data.get('units', 'kilometers')
    try:
        result = client.matrix(sources, targets=targets, costing=costing, units=units)
        return jsonify(result)
    except Exception as exc:
        return jsonify({'error': str(exc)}), 500


if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
