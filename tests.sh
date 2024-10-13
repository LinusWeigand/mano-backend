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



# Pre Register
curl -X POST http://localhost:8000/api/pre-register -H "Content-Type: application/json" -d '{"email": "linus@couchtec.com", "password": "lol"}'
curl -X POST http://localhost/api/pre-register -H "Content-Type: application/json" -d '{"email": "linus@couchtec.com", "password": "lol"}'

# Register
curl -X POST http://localhost:8000/api/register -H "Content-Type: application/json" -d '{"verification_code": "77b6e7dc-dca7-4814-b7ba-f6563ba422e8","email": "linus@couchtec.com"}'

# Login
curl -X POST http://localhost:8000/api/login -H "Content-Type: application/json" -d '{"email": "linus@couchtec.com", "password": "new"}'



# Pre Reset Password
curl -X POST http://localhost:8000/api/pre-reset-password -H "Content-Type: application/json" -d '{"email": "linus@couchtec.com"}'

# Pre Reset Password
curl -X POST http://localhost:8000/api/reset-password -H "Content-Type: application/json" -d '{"email": "linus@couchtec.com", "password": "new", "reset_password_token": "16fd0481-08c9-49c7-92ab-157af89c3632"}'

portfolio1=$(base64 -w 0 holzrausch1.png)
portfolio2=$(base64 -w 0 holzrausch2.png)

curl -X POST http://localhost/api/profile \
-H "Content-Type: multipart/form-data" \
-F "email=linus@couchtec.com" \
-F "name=John Doe" \
-F "craft=Carpentry" \
-F "location=New York" \
-F "website=https://johndoe.com" \
-F "google_ratings=tests" \
-F "instagram=https://instagram.com/johndoe" \
-F "bio=Experienced carpenter with over 10 years in the industry." \
-F "skills=[\"Holzmöbel\",\"Küchen\",\"Badezimmer\"]" \
-F "portfolio_bio=Here is my portfolio showcasing my recent work."


curl -X POST http://localhost/api/search -H "Content-Type: application/json" -d '{"craft": "Zimmerei", "location": "Nürnberg"}'