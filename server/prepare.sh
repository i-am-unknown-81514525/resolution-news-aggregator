 cd server/ || true
 docker run --name postgres_news_agg \
   -e POSTGRES_USER=test \
   -e POSTGRES_PASSWORD=test \
   -p 5432:5432 \
   -d pgvector/pgvector:pg18-trixie
DATABASE_URL=postgres://test:test@localhost:5432 sqlx migrate run
DATABASE_URL=postgres://test:test@localhost:5432 cargo sqlx prepare
docker container remove postgres_news_agg --force