#!/bin/bash

while getopts ":u:" opt; do
  case $opt in
    u)
      username="$OPTARG"
      ;;
    \?)
      echo "Invalid option: -$OPTARG" >&2
      exit 1
      ;;
    :)
      echo "Option -$OPTARG requires an argument." >&2
      exit 1
      ;;
  esac
done

if [ -z "$username" ]; then
  echo "Username is required. Use -u flag."
  exit 1
fi

response=$(curl -s -H 'Content-Type: application/json' \
                  -d "{ \"username\":\"$username\"}" \
                  -X POST \
                  http://localhost:3000/user)

echo "$response"
api_key=$(echo "$response" | grep -o '"api_key":"[^"]*' | awk -F'":"' '{print $2}')
echo "$api_key"
curl -H "api-key:$api_key"  http://localhost:3000/containers

