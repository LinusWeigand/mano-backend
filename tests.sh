# Health Check
curl -X GET http://localhost:8000/api/healthchecker

# List all viewers
curl -X GET http://localhost:8000/api/viewers

# Create Viewer
curl -X POST http://localhost:8000/api/viewers -H "Content-Type: application/json" -d '{"email": "linus@couchtec.com", "hashed": "öasdlfkjsdöflkj", "salt": "ölaksdfjösldkfjaösdlkfj"}'

# Update Viewer
curl -X PATCH http://localhost:8000/api/viewers/1f52f4af-e5b3-4ddd-8c54-d2280fa797c9 -H "Content-Type: application/json" -d '{"email": "linus@couchtec.com", "hashed": "lol", "salt": "omg"}'

# Update Viewer
curl -X PATCH http://localhost:8000/api/viewers/1f52f4af-e5b3-4ddd-8c54-d2280fa797c9 -H "Content-Type: application/json" -d '{"email": "damn", "hashed": "lol", "salt": "omg"}'

# Update Viewer
curl -X PATCH http://localhost:8000/api/viewers/1f52f4af-e5b3-4ddd-8c54-d2280fa797c9 -H "Content-Type: application/json" -d '{"email": "1", "hashed": "2", "salt": "3"}'

# Update Viewer
curl -X PATCH http://localhost:8000/api/viewers/1f52f4af-e5b3-4ddd-8c54-d2280fa797c9 -H "Content-Type: application/json" -d '{}'

# Delete Viewer
curl -X DELETE http://localhost:8000/api/viewers/1f52f4af-e5b3-4ddd-8c54-d2280fa797c9