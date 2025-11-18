#!/bin/bash

curl -d '{"username":"Pepito", "password":"xaqrw", "status": "Active"}' -H "Content-Type: application/json" -X POST http://localhost:8080/v1/users