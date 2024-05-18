docker stop $(docker ps -aq) && docker rm $(docker ps -aq)
docker system prune -a --volumes

