import requests


def test_root():
    resp = requests.get("http://localhost:8080/")
    assert resp.status_code == 200
    assert resp.json() == "Hello, world!"


def test_search_returns_events_within_time_range():
    params = {"start_time": "2025-11-01T08:00:00Z", "end_time": "2025-11-30T18:00:00Z"}

    resp = requests.get("http://localhost:8080/search", params=params)

    assert resp.status_code == 200
    expected_json = {
        "data": {
            "events": [
                {
                    "id": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
                    "title": "Quevedo",
                    "start_date": "2025-11-12",
                    "start_time": "22:00:00",
                    "end_date": "2025-11-12",
                    "end_time": "23:00:00",
                    "min_price": 15.99,
                    "max_price": 39.99,
                }
            ]
        },
        "error": None,
    }

    print(f"Response is {resp.json()}")
    assert resp.json() == expected_json


def test_search_returns_client_error_when_required_param_is_missing():
    params = {"start_time": "2025-11-12T08:00:00Z"}

    resp = requests.get("http://localhost:8080/search", params=params)

    assert resp.status_code == 400
    expected_json = {
        "data": None,
        "error": {"code": "11", "message": "Missing required params"},
    }

    print(f"Response is {resp.json()}")
    assert resp.json() == expected_json


def test_search_returns_server_error_when_something_unexpected_happens():
    params = {"start_time": "2025-11-12T08:00:00Z", "end_time": "2025-11-12T18:00:00Z"}

    resp = requests.get("http://localhost:8080/search", params=params)

    assert resp.status_code == 500
    expected_json = {
        "data": None,
        "error": {"code": "string", "message": "string"},
    }

    print(f"Response is {resp.json()}")
    assert resp.json() == expected_json


def test_ingest_starts_event_data_ingestion():
    resp = requests.patch("http://localhost:8080/ingest")

    print(f"Response status is {resp.status_code}")
    assert resp.status_code == 202
    assert resp.text == ""


if __name__ == "__main__":
    test_root()

    test_search_returns_events_within_time_range()
    test_search_returns_client_error_when_required_param_is_missing()
    # test_search_returns_server_error_when_something_unexpected_happens()

    test_ingest_starts_event_data_ingestion()
    test_search_returns_events_within_time_range()
    test_search_returns_events_within_time_range()
    test_search_returns_client_error_when_required_param_is_missing()
    test_search_returns_events_within_time_range()

    print("All tests passed.")
